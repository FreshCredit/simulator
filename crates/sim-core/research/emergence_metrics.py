"""
Emergence detection metrics for multi-scale analysis.

Implements information-theoretic measures for detecting emergence
from genome to DOCAS dynamics.
"""

from typing import Dict, List, Tuple, Optional
import numpy as np
import pandas as pd
from scipy import stats
from scipy.spatial.distance import pdist, squareform


class TransferEntropyAnalyzer:
    """
    Transfer Entropy analysis for directed information flow.
    
    Measures how much information from genome states at time t
    influences DOCAS states at time t+1, beyond what is already
    contained in DOCAS history.
    
    P0 PERFORMANCE FIX: Vectorized compute_te() method
    - Before: O(bins³ × n) complexity with triple nested loops
    - After: O(n) complexity with numpy histogramdd
    - Speedup: ~1000x for typical bins=10, n=1000
    """
    
    def __init__(self, k: int = 1, bins: int = 10):
        """
        Initialize TE analyzer.
        
        Args:
            k: History length (embedding dimension)
            bins: Number of bins for discretization
        """
        self.k = k
        self.bins = bins
    
    def discretize(self, series: np.ndarray) -> np.ndarray:
        """Discretize continuous series into bins."""
        # Use quantile-based binning for robustness
        percentiles = np.linspace(0, 100, self.bins + 1)
        bin_edges = np.percentile(series, percentiles)
        bin_edges[-1] += 1e-10  # Ensure max value is included
        
        discretized = np.digitize(series, bin_edges[1:-1])
        return discretized.astype(int)
    
    def compute_te_discrete(
        self,
        source: np.ndarray,
        target: np.ndarray
    ) -> float:
        """
        Compute transfer entropy from source to target.
        
        TE(X->Y) = Σ p(y_{t+1}, y_t, x_t) log[p(y_{t+1}|y_t,x_t) / p(y_{t+1}|y_t)]
        
        Args:
            source: Source time series (genome parameters)
            target: Target time series (DOCAS levels)
            
        Returns:
            Transfer entropy value (bits)
        """
        if len(source) != len(target):
            raise ValueError("Source and target must have same length")
        
        n = len(source)
        
        # Discretize
        source_d = self.discretize(source)
        target_d = self.discretize(target)
        
        # P0 PERFORMANCE FIX: Vectorized O(n³) → O(n) transfer entropy calculation
        # Before: Triple nested loop with 10³ = 1000 iterations per data point
        # After: Vectorized numpy operations with histogramdd
        
        # Stack data for joint distribution: [x_t, y_t, y_{t+1}]
        data = np.column_stack([source_d[:-1], target_d[:-1], target_d[1:]])
        
        # Compute joint histogram
        joint_hist, _ = np.histogramdd(
            data,
            bins=[self.bins, self.bins, self.bins],
            range=[[0, self.bins]] * 3
        )
        
        # Normalize to get probabilities
        p_joint = joint_hist / joint_hist.sum()
        
        # Marginal distributions
        p_xy = p_joint.sum(axis=2)  # p(x_t, y_t)
        p_y = p_joint.sum(axis=(0, 2))  # p(y_t)
        p_y_next_given_xy = p_joint / (p_xy[:, :, np.newaxis] + 1e-12)
        p_y_next_given_y = p_joint.sum(axis=0) / (p_y[:, np.newaxis] + 1e-12)
        
        # Mask for valid probabilities
        mask = p_joint > 0
        
        # Compute transfer entropy
        te = np.sum(
            p_joint[mask] * 
            np.log2(
                (p_y_next_given_xy[mask] + 1e-12) / 
                (p_y_next_given_y[np.where(mask)[1], np.where(mask)[2]] + 1e-12)
            )
        )
        
        return max(0, te)  # TE should be non-negative
    
    def compute_te_mixed(
        self,
        genome_df: pd.DataFrame,
        docas_df: pd.DataFrame,
        gene: str,
        neuromodulator: str
    ) -> Dict[str, float]:
        """
        Compute transfer entropy for specific gene→neuromodulator pair.
        
        Args:
            genome_df: DataFrame with genome time series (agent_id, tick)
            docas_df: DataFrame with DOCAS time series (agent_id, tick)
            gene: Gene column name
            neuromodulator: Neuromodulator column name
            
        Returns:
            Dictionary with TE and significance metrics
        """
        # Average across agents for population-level TE
        te_values = []
        
        for agent_id in genome_df.index.get_level_values(0).unique():
            try:
                gene_series = genome_df.loc[agent_id][gene].values
                docas_series = docas_df.loc[agent_id][neuromodulator].values
                
                # Ensure same length
                min_len = min(len(gene_series), len(docas_series))
                gene_series = gene_series[:min_len]
                docas_series = docas_series[:min_len]
                
                te = self.compute_te_discrete(gene_series, docas_series)
                te_values.append(te)
            except KeyError:
                continue
        
        # Compute null model (shuffled)
        te_null_values = []
        for agent_id in genome_df.index.get_level_values(0).unique():
            try:
                gene_series = genome_df.loc[agent_id][gene].values
                docas_series = docas_df.loc[agent_id][neuromodulator].values
                
                # Shuffle source
                np.random.shuffle(gene_series)
                
                min_len = min(len(gene_series), len(docas_series))
                gene_series = gene_series[:min_len]
                docas_series = docas_series[:min_len]
                
                te_null = self.compute_te_discrete(gene_series, docas_series)
                te_null_values.append(te_null)
            except KeyError:
                continue
        
        # Statistical significance
        t_stat, p_value = stats.ttest_ind(te_values, te_null_values)
        
        return {
            'transfer_entropy': np.mean(te_values),
            'te_std': np.std(te_values),
            'te_null_mean': np.mean(te_null_values),
            'te_null_std': np.std(te_null_values),
            't_statistic': t_stat,
            'p_value': p_value,
            'significant': p_value < 0.05,
            'n_agents': len(te_values)
        }
    
    def compute_te_matrix(
        self,
        genome_df: pd.DataFrame,
        docas_df: pd.DataFrame
    ) -> pd.DataFrame:
        """
        Compute full transfer entropy matrix for all gene→neuromodulator pairs.
        
        Args:
            genome_df: Genome time series data
            docas_df: DOCAS time series data
            
        Returns:
            DataFrame with TE values (genes × neuromodulators)
        """
        genes = [c for c in genome_df.columns if c != 'tick']
        neuromodulators = [c for c in docas_df.columns if c != 'tick']
        
        results = {}
        
        for gene in genes:
            for nm in neuromodulators:
                key = f"{gene}→{nm}"
                result = self.compute_te_mixed(genome_df, docas_df, gene, nm)
                results[key] = result
        
        # Convert to DataFrame
        te_df = pd.DataFrame(results).T
        return te_df


class MultiScaleEntropyAnalyzer:
    """
    Multi-Scale Entropy (MSE) analysis for complexity quantification.
    
    Measures complexity of DOCAS dynamics across multiple time scales.
    High MSE indicates emergent structure not reducible to simple components.
    """
    
    def __init__(self, max_scale: int = 20, m: int = 2, r: float = 0.15):
        """
        Initialize MSE analyzer.
        
        Args:
            max_scale: Maximum coarse-graining scale
            m: Embedding dimension for sample entropy
            r: Tolerance (as fraction of std)
        """
        self.max_scale = max_scale
        self.m = m
        self.r = r
    
    def coarse_grain(self, series: np.ndarray, scale: int) -> np.ndarray:
        """
        Coarse-grain time series at given scale.
        
        Args:
            series: Input time series
            scale: Coarse-graining factor
            
        Returns:
            Coarse-grained series
        """
        n = len(series) // scale
        cg = np.array([np.mean(series[i*scale:(i+1)*scale]) for i in range(n)])
        return cg
    
    def sample_entropy(self, series: np.ndarray) -> float:
        """
        Compute sample entropy of series.
        
        Args:
            series: Input time series
            
        Returns:
            Sample entropy value
        """
        N = len(series)
        if N <= self.m:
            return 0.0
        
        r_val = self.r * np.std(series)
        
        # Count matching templates of length m and m+1
        def count_matches(u, m, r):
            count = 0
            for i in range(len(u) - m):
                for j in range(i + 1, len(u) - m):
                    if np.max(np.abs(u[i:i+m] - u[j:j+m])) < r:
                        count += 1
            return count
        
        A = count_matches(series, self.m + 1, r_val)
        B = count_matches(series, self.m, r_val)
        
        if B == 0:
            return 0.0
        
        return -np.log(A / B) if A > 0 else 0.0
    
    def compute_mse(self, series: np.ndarray) -> Dict[int, float]:
        """
        Compute multi-scale entropy profile.
        
        Args:
            series: Input time series
            
        Returns:
            Dictionary mapping scale to entropy value
        """
        mse_profile = {}
        
        for scale in range(1, self.max_scale + 1):
            cg_series = self.coarse_grain(series, scale)
            
            if len(cg_series) > self.m + 1:
                se = self.sample_entropy(cg_series)
                mse_profile[scale] = se
            else:
                mse_profile[scale] = np.nan
        
        return mse_profile
    
    def compute_area_under_curve(self, mse_profile: Dict[int, float]) -> float:
        """
        Compute area under MSE curve (composite complexity index).
        
        Args:
            mse_profile: Dictionary of scale→entropy values
            
        Returns:
            Area under curve
        """
        scales = sorted(mse_profile.keys())
        values = [mse_profile[s] for s in scales]
        
        # Remove NaN values
        valid = [(s, v) for s, v in zip(scales, values) if not np.isnan(v)]
        if not valid:
            return 0.0
        
        scales, values = zip(*valid)
        
        # Simple trapezoidal integration (NumPy 2.x compatible)
        if hasattr(np, "trapezoid"):
            area = float(np.trapezoid(values, scales))
        elif hasattr(np, "trapz"):
            area = float(np.trapz(values, scales))
        else:
            dx = np.diff(scales)
            area = float(np.sum(0.5 * (np.asarray(values)[:-1] + np.asarray(values)[1:]) * dx))
        return area
    
    def analyze_population(
        self,
        docas_df: pd.DataFrame,
        neuromodulator: str
    ) -> Dict[str, any]:
        """
        Analyze MSE for entire population.
        
        Args:
            docas_df: DOCAS time series (MultiIndex: agent_id, tick)
            neuromodulator: Neuromodulator to analyze
            
        Returns:
            Population-level MSE statistics
        """
        mse_profiles = []
        auc_values = []
        
        for agent_id in docas_df.index.get_level_values(0).unique():
            try:
                series = docas_df.loc[agent_id][neuromodulator].values
                
                if len(series) < self.max_scale:
                    continue
                
                mse = self.compute_mse(series)
                mse_profiles.append(mse)
                
                auc = self.compute_area_under_curve(mse)
                auc_values.append(auc)
            except KeyError:
                continue
        
        # Aggregate profiles
        scales = range(1, self.max_scale + 1)
        mean_profile = {}
        std_profile = {}
        
        for scale in scales:
            values = [p.get(scale, np.nan) for p in mse_profiles]
            values = [v for v in values if not np.isnan(v)]
            if values:
                mean_profile[scale] = np.mean(values)
                std_profile[scale] = np.std(values)
            else:
                mean_profile[scale] = np.nan
                std_profile[scale] = np.nan
        
        return {
            'mean_profile': mean_profile,
            'std_profile': std_profile,
            'mean_auc': np.mean(auc_values),
            'std_auc': np.std(auc_values),
            'n_agents': len(mse_profiles)
        }


class EmergenceDetector:
    """
    Composite emergence detector combining multiple metrics.
    
    Uses TE, MSE, and correlation structure to detect emergent properties
    at the genome→neurobiology boundary.
    """
    
    def __init__(self):
        self.te_analyzer = TransferEntropyAnalyzer()
        self.mse_analyzer = MultiScaleEntropyAnalyzer()
    
    def detect_emergence(
        self,
        genome_df: pd.DataFrame,
        docas_df: pd.DataFrame,
        threshold_te: float = 0.1,
        threshold_mse_auc: float = 10.0
    ) -> Dict[str, any]:
        """
        Detect emergence using multiple criteria.
        
        Emergence criteria:
        1. Significant transfer entropy (information flow)
        2. High MSE complexity (irreducible dynamics)
        3. Non-linear correlation structure
        
        Args:
            genome_df: Genome time series
            docas_df: DOCAS time series
            threshold_te: Minimum TE for significance
            threshold_mse_auc: Minimum MSE area for complexity
            
        Returns:
            Emergence detection report
        """
        results = {
            'transfer_entropy': {},
            'multi_scale_entropy': {},
            'emergence_detected': False,
            'emergence_score': 0.0,
        }
        
        # Compute TE matrix
        te_matrix = self.te_analyzer.compute_te_matrix(genome_df, docas_df)
        results['transfer_entropy'] = te_matrix
        
        # Count significant TE pathways
        significant_te = te_matrix[te_matrix['significant'] == True]
        results['n_significant_pathways'] = len(significant_te)
        results['mean_te'] = te_matrix['transfer_entropy'].mean()
        
        # Compute MSE for each neuromodulator
        nm_columns = [c for c in docas_df.columns if c != 'tick']
        
        for nm in nm_columns:
            mse_result = self.mse_analyzer.analyze_population(docas_df, nm)
            results['multi_scale_entropy'][nm] = mse_result
        
        # Composite emergence score
        te_score = min(1.0, results['mean_te'] / threshold_te)
        
        mse_scores = [
            r['mean_auc'] / threshold_mse_auc 
            for r in results['multi_scale_entropy'].values()
        ]
        mse_score = np.mean(mse_scores) if mse_scores else 0
        
        results['emergence_score'] = 0.5 * te_score + 0.5 * min(1.0, mse_score)
        results['emergence_detected'] = results['emergence_score'] > 0.5
        
        return results


if __name__ == "__main__":
    # Example usage with synthetic data
    np.random.seed(42)
    
    # Generate synthetic genome data
    n_agents = 10
    n_ticks = 1000
    
    genome_data = []
    for agent_id in range(n_agents):
        for tick in range(n_ticks):
            genome_data.append({
                'agent_id': agent_id,
                'tick': tick,
                'dopamine_clearance': np.random.randn(),
                'oxytocin_receptor': np.random.randn(),
            })
    
    genome_df = pd.DataFrame(genome_data).set_index(['agent_id', 'tick'])
    
    # Generate synthetic DOCAS data (with some structure from genome)
    docas_data = []
    for agent_id in range(n_agents):
        dc = genome_df.loc[agent_id]['dopamine_clearance'].values
        for i, tick in enumerate(range(n_ticks)):
            docas_data.append({
                'agent_id': agent_id,
                'tick': tick,
                'dopamine': dc[i] * 0.5 + np.random.randn(),
                'oxytocin': np.random.randn(),
            })
    
    docas_df = pd.DataFrame(docas_data).set_index(['agent_id', 'tick'])
    
    # Detect emergence
    detector = EmergenceDetector()
    report = detector.detect_emergence(genome_df, docas_df)
    
    print("Emergence Detection Report")
    print(f"Score: {report['emergence_score']:.3f}")
    print(f"Detected: {report['emergence_detected']}")
    print(f"Significant TE pathways: {report['n_significant_pathways']}")
