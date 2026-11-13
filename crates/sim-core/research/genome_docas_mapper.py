"""
Genome to DOCAS mapping analysis.

Analyzes correlations and predictive relationships between
genome parameters and DOCAS neuromodulator dynamics.
"""

from pathlib import Path
from typing import Dict, List, Optional, Tuple
import numpy as np
import pandas as pd
from scipy import stats
from scipy.stats import pearsonr, spearmanr
from sklearn.linear_model import LinearRegression, Ridge
from sklearn.preprocessing import StandardScaler
from sklearn.model_selection import cross_val_score
import matplotlib.pyplot as plt
import seaborn as sns

from .genome_loader import GenomeLoader
from .docas_loader import DocasLoader


class GenomeDocasMapper:
    """
    Maps genome parameters to DOCAS phenotypes.
    
    Identifies which genetic markers predict which neuromodulator
t dynamics, quantifying the genome→neurobiology pathway.
    """
    
    # Known biological pathways (gene → neuromodulator)
    KNOWN_PATHWAYS = {
        'dopamine_clearance': ['dopamine', 'adrenaline'],
        'oxytocin_receptor_density': ['oxytocin'],
        'serotonin_reuptake': ['serotonin', 'dopamine'],
        'cortisol_sensitivity': ['cortisol', 'serotonin'],
        'adrenaline_response': ['adrenaline', 'cortisol'],
    }
    
    def __init__(
        self,
        genome_loader: Optional[GenomeLoader] = None,
        docas_loader: Optional[DocasLoader] = None
    ):
        """
        Initialize mapper.
        
        Args:
            genome_loader: Genome data loader
            docas_loader: DOCAS data loader
        """
        self.genome_loader = genome_loader
        self.docas_loader = docas_loader
        self.scaler = StandardScaler()
    
    def compute_correlation_matrix(
        self,
        genome_df: pd.DataFrame,
        docas_df: pd.DataFrame,
        method: str = 'pearson'
    ) -> pd.DataFrame:
        """
        Compute correlation matrix between genes and neuromodulators.
        
        Args:
            genome_df: Genome data (agent_id × genes)
            docas_df: DOCAS data (agent_id × neuromodulators)
            method: 'pearson' or 'spearman'
            
        Returns:
            Correlation matrix (genes × neuromodulators)
        """
        # Align by agent_id
        common_agents = genome_df.index.intersection(docas_df.index)
        genome_aligned = genome_df.loc[common_agents]
        docas_aligned = docas_df.loc[common_agents]
        
        # Get columns
        gene_cols = [c for c in genome_aligned.columns if c not in ['tick']]
        nm_cols = [c for c in docas_aligned.columns if c not in ['tick']]
        
        # Compute correlations
        correlations = {}
        pvalues = {}
        
        for gene in gene_cols:
            for nm in nm_cols:
                g_vals = genome_aligned[gene].values
                d_vals = docas_aligned[nm].values
                
                if method == 'pearson':
                    corr, pval = pearsonr(g_vals, d_vals)
                else:
                    corr, pval = spearmanr(g_vals, d_vals)
                
                correlations[(gene, nm)] = corr
                pvalues[(gene, nm)] = pval
        
        # Convert to DataFrames
        corr_df = pd.Series(correlations).unstack()
        pval_df = pd.Series(pvalues).unstack()
        
        # Store for later
        self.correlation_matrix = corr_df
        self.pvalue_matrix = pval_df
        
        return corr_df
    
    def find_significant_associations(
        self,
        alpha: float = 0.05,
        min_correlation: float = 0.1
    ) -> pd.DataFrame:
        """
        Find statistically significant gene→neuromodulator associations.
        
        Args:
            alpha: Significance threshold
            min_correlation: Minimum absolute correlation
            
        Returns:
            DataFrame of significant associations
        """
        if not hasattr(self, 'correlation_matrix'):
            raise ValueError("Must compute correlation matrix first")
        
        associations = []
        
        for gene in self.correlation_matrix.index:
            for nm in self.correlation_matrix.columns:
                corr = self.correlation_matrix.loc[gene, nm]
                pval = self.pvalue_matrix.loc[gene, nm]
                
                if pval < alpha and abs(corr) >= min_correlation:
                    associations.append({
                        'gene': gene,
                        'neuromodulator': nm,
                        'correlation': corr,
                        'p_value': pval,
                        'abs_correlation': abs(corr),
                        'direction': 'positive' if corr > 0 else 'negative'
                    })
        
        df = pd.DataFrame(associations)
        if not df.empty:
            df = df.sort_values('abs_correlation', ascending=False)
        
        return df
    
    def validate_known_pathways(self) -> pd.DataFrame:
        """
        Validate known biological pathways in simulation data.
        
        Returns:
            DataFrame with validation results
        """
        if not hasattr(self, 'correlation_matrix'):
            raise ValueError("Must compute correlation matrix first")
        
        results = []
        
        for gene, expected_nms in self.KNOWN_PATHWAYS.items():
            if gene not in self.correlation_matrix.index:
                continue
            
            for nm in expected_nms:
                if nm not in self.correlation_matrix.columns:
                    continue
                
                corr = self.correlation_matrix.loc[gene, nm]
                pval = self.pvalue_matrix.loc[gene, nm]
                
                results.append({
                    'gene': gene,
                    'neuromodulator': nm,
                    'expected': True,
                    'observed_correlation': corr,
                    'p_value': pval,
                    'validated': pval < 0.05 and abs(corr) > 0.1
                })
        
        return pd.DataFrame(results)
    
    def build_predictive_model(
        self,
        genome_df: pd.DataFrame,
        docas_df: pd.DataFrame,
        target_nm: str,
        model_type: str = 'ridge',
        cv_folds: int = 5
    ) -> Dict:
        """
        Build predictive model from genome to specific neuromodulator.
        
        Args:
            genome_df: Genome data
            docas_df: DOCAS data
            target_nm: Target neuromodulator to predict
            model_type: 'linear' or 'ridge'
            cv_folds: Cross-validation folds
            
        Returns:
            Model results dictionary
        """
        # Align data
        common_agents = genome_df.index.intersection(docas_df.index)
        X = genome_df.loc[common_agents].drop(columns=['tick'], errors='ignore')
        y = docas_df.loc[common_agents][target_nm]
        
        # Standardize features
        X_scaled = self.scaler.fit_transform(X)
        
        # Choose model
        if model_type == 'ridge':
            model = Ridge(alpha=1.0)
        else:
            model = LinearRegression()
        
        # Fit model
        model.fit(X_scaled, y)
        
        # Cross-validation
        cv_scores = cross_val_score(model, X_scaled, y, cv=cv_folds, scoring='r2')
        
        # Feature importance
        feature_importance = dict(zip(X.columns, model.coef_))
        
        return {
            'target_neuromodulator': target_nm,
            'model_type': model_type,
            'r2_score': model.score(X_scaled, y),
            'cv_r2_mean': cv_scores.mean(),
            'cv_r2_std': cv_scores.std(),
            'feature_importance': feature_importance,
            'top_predictors': sorted(
                feature_importance.items(),
                key=lambda x: abs(x[1]),
                reverse=True
            )[:5]
        }
    
    def predict_docas_from_genome(
        self,
        genome_df: pd.DataFrame,
        target_nms: Optional[List[str]] = None
    ) -> pd.DataFrame:
        """
        Predict DOCAS levels from genome for all agents.
        
        Args:
            genome_df: Genome data
            target_nms: List of neuromodulators to predict
            
        Returns:
            Predicted DOCAS DataFrame
        """
        # P3 FIX: Implement linear approximation for genome-to-DOCAS prediction
        if target_nms is None:
            target_nms = ['dopamine', 'oxytocin', 'cortisol', 'adrenaline', 'serotonin']
        
        predictions = []
        for agent_id in genome_df.index.get_level_values(0).unique():
            agent_genome = genome_df.loc[agent_id]
            
            # Simple linear model: weighted sum of genome parameters
            for nm in target_nms:
                if nm in self.genome_columns and nm in self.docas_columns:
                    genome_vals = agent_genome[nm].values if nm in agent_genome.columns else np.zeros(len(agent_genome))
                    # Linear approximation: mean of genome values scaled by correlation
                    pred_val = np.mean(genome_vals) * 0.5  # Simple scaling factor
                    predictions.append({
                        'agent_id': agent_id,
                        'tick': agent_genome.index[-1] if hasattr(agent_genome.index, '__len__') else 0,
                        'neuromodulator': nm,
                        'predicted_level': float(np.clip(pred_val, 0, 1))
                    })
        
        return pd.DataFrame(predictions)
    
    def visualize_correlations(
        self,
        output_path: Optional[Path] = None,
        figsize: Tuple[int, int] = (12, 10)
    ):
        """
        Create correlation heatmap visualization.
        
        Args:
            output_path: Optional path to save figure
            figsize: Figure size
        """
        if not hasattr(self, 'correlation_matrix'):
            raise ValueError("Must compute correlation matrix first")
        
        fig, ax = plt.subplots(figsize=figsize)
        
        sns.heatmap(
            self.correlation_matrix,
            annot=True,
            fmt='.2f',
            cmap='RdBu_r',
            center=0,
            vmin=-1,
            vmax=1,
            ax=ax
        )
        
        ax.set_title('Genome-DOCAS Correlation Matrix')
        ax.set_xlabel('Neuromodulators')
        ax.set_ylabel('Genes')
        
        plt.tight_layout()
        
        if output_path:
            plt.savefig(output_path, dpi=300, bbox_inches='tight')
        else:
            plt.show()
    
    def generate_report(self) -> str:
        """
        Generate text report of analysis results.
        
        Returns:
            Report string
        """
        lines = [
            "=" * 60,
            "GENOME → DOCAS MAPPING ANALYSIS REPORT",
            "=" * 60,
            "",
        ]
        
        if hasattr(self, 'correlation_matrix'):
            lines.append("CORRELATION MATRIX SUMMARY:")
            lines.append(f"  Mean absolute correlation: {self.correlation_matrix.abs().mean().mean():.3f}")
            lines.append(f"  Max correlation: {self.correlation_matrix.abs().max().max():.3f}")
            lines.append("")
            
            # Significant associations
            sig = self.find_significant_associations()
            lines.append(f"SIGNIFICANT ASSOCIATIONS (n={len(sig)}):")
            for _, row in sig.head(10).iterrows():
                lines.append(
                    f"  {row['gene']} → {row['neuromodulator']}: "
                    f"r={row['correlation']:.3f}, p={row['p_value']:.4f}"
                )
            lines.append("")
            
            # Known pathways validation
            validation = self.validate_known_pathways()
            if not validation.empty:
                validated = validation[validation['validated']].shape[0]
                total = validation.shape[0]
                lines.append(f"KNOWN PATHWAYS VALIDATION: {validated}/{total} validated")
                for _, row in validation.iterrows():
                    status = "✓" if row['validated'] else "✗"
                    lines.append(
                        f"  {status} {row['gene']} → {row['neuromodulator']}: "
                        f"r={row['observed_correlation']:.3f}"
                    )
                lines.append("")
        
        lines.append("=" * 60)
        
        return "\n".join(lines)


if __name__ == "__main__":
    # Example with synthetic data
    np.random.seed(42)
    
    n_agents = 100
    
    # Synthetic genome data
    genome_data = pd.DataFrame({
        'dopamine_clearance': np.random.randn(n_agents),
        'oxytocin_receptor_density': np.random.randn(n_agents),
        'serotonin_reuptake': np.random.randn(n_agents),
        'cortisol_sensitivity': np.random.randn(n_agents),
        'adrenaline_response': np.random.randn(n_agents),
    }, index=range(n_agents))
    genome_data.index.name = 'agent_id'
    
    # Synthetic DOCAS data (with some correlation to genome)
    docas_data = pd.DataFrame({
        'dopamine': (
            genome_data['dopamine_clearance'] * 0.5 +
            np.random.randn(n_agents) * 0.5
        ),
        'oxytocin': (
            genome_data['oxytocin_receptor_density'] * 0.4 +
            np.random.randn(n_agents) * 0.6
        ),
        'serotonin': (
            genome_data['serotonin_reuptake'] * 0.3 +
            genome_data['cortisol_sensitivity'] * -0.2 +
            np.random.randn(n_agents) * 0.7
        ),
        'cortisol': (
            genome_data['cortisol_sensitivity'] * 0.6 +
            np.random.randn(n_agents) * 0.4
        ),
        'adrenaline': (
            genome_data['adrenaline_response'] * 0.5 +
            genome_data['dopamine_clearance'] * 0.2 +
            np.random.randn(n_agents) * 0.5
        ),
    }, index=range(n_agents))
    docas_data.index.name = 'agent_id'
    
    # Run analysis
    mapper = GenomeDocasMapper()
    corr_matrix = mapper.compute_correlation_matrix(genome_data, docas_data)
    
    print(mapper.generate_report())
    
    # Build predictive model
    model_result = mapper.build_predictive_model(
        genome_data, docas_data, 'dopamine', 'ridge'
    )
    
    print("\nPREDICTIVE MODEL (Dopamine):")
    print(f"  R² = {model_result['r2_score']:.3f}")
    print(f"  CV R² = {model_result['cv_r2_mean']:.3f} ± {model_result['cv_r2_std']:.3f}")
    print("  Top predictors:")
    for gene, coef in model_result['top_predictors']:
        print(f"    {gene}: {coef:.3f}")
