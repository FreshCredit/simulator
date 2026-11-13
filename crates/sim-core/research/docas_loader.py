"""
DOCAS time series loader for Layer 3 analysis.

Loads DOCAS (Dopamine, Oxytocin, Cortisol, Adrenaline, Serotonin) 
time series from Arrow IPC format.
"""

from pathlib import Path
from typing import Dict, List, Optional, Tuple
import numpy as np
import pandas as pd
import pyarrow as pa
import pyarrow.ipc as ipc


class DocasLoader:
    """Load DOCAS time series from Arrow IPC files."""
    
    DOCAS_FIELDS = [
        'dopamine',
        'oxytocin', 
        'cortisol',
        'adrenaline',
        'serotonin'
    ]
    
    def __init__(self, data_dir: Path):
        """
        Initialize DOCAS loader.
        
        Args:
            data_dir: Directory containing docas_{tick}.arrow files
        """
        self.data_dir = Path(data_dir)
        self._cache: Dict[int, pd.DataFrame] = {}
    
    def load_tick(self, tick: int) -> pd.DataFrame:
        """
        Load DOCAS data for specific tick.
        
        Args:
            tick: Simulation tick number
            
        Returns:
            DataFrame with agent_id as index and DOCAS levels as columns
        """
        if tick in self._cache:
            return self._cache[tick]
        
        file_path = self.data_dir / f"docas_{tick}.arrow"
        if not file_path.exists():
            raise FileNotFoundError(f"DOCAS file not found: {file_path}")
        
        df = self._load_arrow(file_path)
        self._cache[tick] = df
        return df
    
    def _load_arrow(self, file_path: Path) -> pd.DataFrame:
        """Load Arrow IPC file."""
        with open(file_path, 'rb') as f:
            reader = ipc.RecordBatchFileReader(f)
            table = reader.read_all()
        
        df = table.to_pandas()
        
        # Ensure agent_id is index
        if 'agent_id' in df.columns:
            df = df.set_index('agent_id')
        
        return df
    
    def load_time_series(
        self,
        agent_id: int,
        ticks: Optional[List[int]] = None
    ) -> pd.DataFrame:
        """
        Load DOCAS time series for specific agent.
        
        Args:
            agent_id: Agent identifier
            ticks: Optional list of ticks (loads all available if None)
            
        Returns:
            DataFrame with tick as index and DOCAS levels as columns
        """
        if ticks is None:
            ticks = self.get_available_ticks()
        
        data = []
        for tick in ticks:
            try:
                df = self.load_tick(tick)
                if agent_id in df.index:
                    row = df.loc[agent_id].copy()
                    row['tick'] = tick
                    data.append(row)
            except FileNotFoundError:
                continue
        
        if not data:
            return pd.DataFrame(columns=self.DOCAS_FIELDS)
        
        result = pd.DataFrame(data)
        result = result.set_index('tick')
        result = result.sort_index()
        
        return result
    
    def load_batch_time_series(
        self,
        agent_ids: List[int],
        ticks: Optional[List[int]] = None
    ) -> pd.DataFrame:
        """
        Load DOCAS time series for multiple agents.
        
        Args:
            agent_ids: List of agent identifiers
            ticks: Optional list of ticks
            
        Returns:
            DataFrame with MultiIndex (agent_id, tick)
        """
        if ticks is None:
            ticks = self.get_available_ticks()
        
        frames = []
        for tick in ticks:
            try:
                df = self.load_tick(tick)
                df = df[df.index.isin(agent_ids)].copy()
                df['tick'] = tick
                frames.append(df)
            except FileNotFoundError:
                continue
        
        if not frames:
            return pd.DataFrame(columns=self.DOCAS_FIELDS)
        
        combined = pd.concat(frames)
        combined = combined.set_index('tick', append=True)
        combined = combined.reorder_levels(['agent_id', 'tick'])
        
        return combined
    
    def get_available_ticks(self) -> List[int]:
        """Get list of available tick numbers from directory."""
        ticks = []
        for file_path in self.data_dir.glob("docas_*.arrow"):
            try:
                tick = int(file_path.stem.split('_')[1])
                ticks.append(tick)
            except (IndexError, ValueError):
                continue
        return sorted(ticks)
    
    def compute_summary_statistics(self, tick: int) -> pd.DataFrame:
        """
        Compute summary statistics for DOCAS at given tick.
        
        Args:
            tick: Simulation tick
            
        Returns:
            DataFrame with statistics for each neuromodulator
        """
        df = self.load_tick(tick)
        
        stats = df[self.DOCAS_FIELDS].agg(['mean', 'std', 'min', 'max', 'median'])
        return stats
    
    def compute_temporal_dynamics(
        self,
        agent_ids: Optional[List[int]] = None,
        window_size: int = 100
    ) -> pd.DataFrame:
        """
        Compute temporal dynamics (velocity, acceleration) of DOCAS.
        
        Args:
            agent_ids: Optional filter for specific agents
            window_size: Window for smoothing
            
        Returns:
            DataFrame with velocity and acceleration metrics
        """
        ticks = self.get_available_ticks()
        
        if agent_ids is None:
            # Load first tick to get all agents
            first_tick_df = self.load_tick(ticks[0])
            agent_ids = first_tick_df.index.tolist()
        
        results = []
        
        for agent_id in agent_ids:
            ts = self.load_time_series(agent_id, ticks)
            
            if len(ts) < 3:
                continue
            
            for docas in self.DOCAS_FIELDS:
                series = ts[docas]
                
                # Compute velocity (first derivative)
                velocity = series.diff().mean()
                
                # Compute acceleration (second derivative)
                acceleration = series.diff().diff().mean()
                
                # Compute volatility (std of velocity)
                volatility = series.diff().std()
                
                # Compute range
                range_val = series.max() - series.min()
                
                results.append({
                    'agent_id': agent_id,
                    'neuromodulator': docas,
                    'mean_level': series.mean(),
                    'velocity': velocity,
                    'acceleration': acceleration,
                    'volatility': volatility,
                    'range': range_val,
                })
        
        return pd.DataFrame(results)
    
    def extract_events(
        self,
        agent_id: int,
        threshold_std: float = 2.0,
        min_duration: int = 5
    ) -> pd.DataFrame:
        """
        Extract anomalous events from DOCAS time series.
        
        Args:
            agent_id: Agent identifier
            threshold_std: Threshold in standard deviations
            min_duration: Minimum event duration in ticks
            
        Returns:
            DataFrame with event start, end, type, magnitude
        """
        ts = self.load_time_series(agent_id)
        
        events = []
        
        for docas in self.DOCAS_FIELDS:
            series = ts[docas]
            mean = series.mean()
            std = series.std()
            
            # Detect anomalies
            is_anomaly = np.abs(series - mean) > (threshold_std * std)
            
            # Find contiguous regions
            in_event = False
            event_start = None
            
            for tick, anomalous in is_anomaly.items():
                if anomalous and not in_event:
                    event_start = tick
                    in_event = True
                elif not anomalous and in_event:
                    event_end = tick
                    duration = event_end - event_start
                    
                    if duration >= min_duration:
                        event_data = series.loc[event_start:event_end]
                        events.append({
                            'agent_id': agent_id,
                            'neuromodulator': docas,
                            'start_tick': event_start,
                            'end_tick': event_end,
                            'duration': duration,
                            'max_deviation': (event_data - mean).abs().max() / std,
                            'mean_level': event_data.mean(),
                        })
                    
                    in_event = False
        
        return pd.DataFrame(events)


if __name__ == "__main__":
    # Example usage
    loader = DocasLoader(Path("./data/docas"))
    
    # Load specific tick
    docas = loader.load_tick(10000)
    print(f"Loaded {len(docas)} DOCAS records")
    print(docas.head())
    
    # Get statistics
    stats = loader.compute_summary_statistics(10000)
    print("\nDOCAS statistics:")
    print(stats)
