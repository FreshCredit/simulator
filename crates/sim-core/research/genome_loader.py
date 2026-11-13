"""
Genome data loader for Layer 3 analysis.

Loads genome parameters from Rust simulation binary output format.
"""

import struct
from pathlib import Path
from typing import Dict, List, Optional, BinaryIO
import numpy as np
import pandas as pd


class GenomeLoader:
    """Load genome data from Rust simulation binary format."""
    
    # Genome struct layout (from sim-core/src/genome.rs)
    GENOME_SIZE_BYTES = 48  # 12 f32 values * 4 bytes
    
    GENOME_FIELDS = [
        # Neuromodulator genes
        'dopamine_clearance',      # COMT proxy
        'oxytocin_receptor_density',  # OXTR proxy
        'serotonin_reuptake',      # 5-HTTLPR proxy
        'cortisol_sensitivity',    # CRHR1 proxy
        'adrenaline_response',     # ADRA2B proxy
        # Social genes
        'social_orientation',      # DRD4 proxy
        'risk_tolerance_base',     # DAT1 proxy
        # Temporal genes
        'circadian_rhythm',        # CLOCK proxy
        'sleep_preference',        # PER3 proxy
        # Metabolic genes
        'metabolic_rate',          # UCP2 proxy
        # Development genes
        'maturation_rate',         # IGF1 proxy
        'longevity_bias',          # FOXO3 proxy
    ]
    
    def __init__(self, data_dir: Path):
        """
        Initialize genome loader.
        
        Args:
            data_dir: Directory containing genome_{tick}.bin files
        """
        self.data_dir = Path(data_dir)
        self._cache: Dict[int, pd.DataFrame] = {}
    
    def load_tick(self, tick: int) -> pd.DataFrame:
        """
        Load genome data for specific tick.
        
        Args:
            tick: Simulation tick number
            
        Returns:
            DataFrame with agent_id as index and genome fields as columns
        """
        if tick in self._cache:
            return self._cache[tick]
        
        file_path = self.data_dir / f"genome_{tick}.bin"
        if not file_path.exists():
            raise FileNotFoundError(f"Genome file not found: {file_path}")
        
        df = self._load_binary(file_path)
        self._cache[tick] = df
        return df
    
    def _load_binary(self, file_path: Path) -> pd.DataFrame:
        """Load binary genome file."""
        data = []
        agent_ids = []
        
        with open(file_path, 'rb') as f:
            # Read header: number of agents (u64)
            header = f.read(8)
            if not header:
                return pd.DataFrame(columns=self.GENOME_FIELDS)
            
            n_agents = struct.unpack('<Q', header)[0]
            
            # Read each agent's genome
            for i in range(n_agents):
                # Agent ID (u64)
                agent_id_bytes = f.read(8)
                if not agent_id_bytes:
                    break
                agent_id = struct.unpack('<Q', agent_id_bytes)[0]
                agent_ids.append(agent_id)
                
                # Genome data (12 f32 values)
                genome_bytes = f.read(self.GENOME_SIZE_BYTES)
                if len(genome_bytes) != self.GENOME_SIZE_BYTES:
                    break
                
                genome = struct.unpack('<' + 'f' * 12, genome_bytes)
                data.append(genome)
        
        df = pd.DataFrame(data, columns=self.GENOME_FIELDS, index=agent_ids)
        df.index.name = 'agent_id'
        return df
    
    def load_time_series(
        self, 
        ticks: List[int],
        agent_ids: Optional[List[int]] = None
    ) -> pd.DataFrame:
        """
        Load genome time series for multiple ticks.
        
        Args:
            ticks: List of tick numbers
            agent_ids: Optional filter for specific agents
            
        Returns:
            DataFrame with MultiIndex (agent_id, tick)
        """
        frames = []
        
        for tick in ticks:
            df = self.load_tick(tick)
            df['tick'] = tick
            
            if agent_ids is not None:
                df = df[df.index.isin(agent_ids)]
            
            frames.append(df)
        
        combined = pd.concat(frames)
        combined = combined.set_index('tick', append=True)
        combined = combined.reorder_levels(['agent_id', 'tick'])
        
        return combined
    
    def get_available_ticks(self) -> List[int]:
        """Get list of available tick numbers from directory."""
        ticks = []
        for file_path in self.data_dir.glob("genome_*.bin"):
            try:
                tick = int(file_path.stem.split('_')[1])
                ticks.append(tick)
            except (IndexError, ValueError):
                continue
        return sorted(ticks)
    
    def compute_genome_statistics(self, tick: int) -> pd.Series:
        """
        Compute summary statistics for genome at given tick.
        
        Args:
            tick: Simulation tick
            
        Returns:
            Series with mean, std, min, max for each gene
        """
        df = self.load_tick(tick)
        
        stats = {}
        for col in self.GENOME_FIELDS:
            stats[f'{col}_mean'] = df[col].mean()
            stats[f'{col}_std'] = df[col].std()
            stats[f'{col}_min'] = df[col].min()
            stats[f'{col}_max'] = df[col].max()
        
        return pd.Series(stats)
    
    def correlate_genome_docas(
        self, 
        genome_tick: int,
        docas_data: pd.DataFrame
    ) -> pd.DataFrame:
        """
        Compute correlation between genome and DOCAS levels.
        
        Args:
            genome_tick: Tick for genome data
            docas_data: DataFrame with DOCAS levels (indexed by agent_id)
            
        Returns:
            Correlation matrix
        """
        genome = self.load_tick(genome_tick)
        
        # Align indices
        common_agents = genome.index.intersection(docas_data.index)
        genome_aligned = genome.loc[common_agents]
        docas_aligned = docas_data.loc[common_agents]
        
        # Compute correlations
        correlations = {}
        for gene in self.GENOME_FIELDS:
            for docas in docas_aligned.columns:
                corr = np.corrcoef(
                    genome_aligned[gene].values,
                    docas_aligned[docas].values
                )[0, 1]
                correlations[(gene, docas)] = corr
        
        corr_df = pd.Series(correlations).unstack()
        return corr_df


if __name__ == "__main__":
    # Example usage
    loader = GenomeLoader(Path("./data/genomes"))
    
    # Load specific tick
    genomes = loader.load_tick(10000)
    print(f"Loaded {len(genomes)} genomes")
    print(genomes.head())
    
    # Get statistics
    stats = loader.compute_genome_statistics(10000)
    print("\nGenome statistics:")
    print(stats)
