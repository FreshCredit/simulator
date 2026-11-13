#!/usr/bin/env python3
"""
Main pipeline for Layer 3 Multi-Scale Emergence Analysis.

This script runs the complete analysis pipeline from genome/DOCAS data
to emergence detection and reporting.

Usage:
    python run_analysis.py --data-dir ./data/sims --output ./results
    
    python run_analysis.py --genome-dir ./data/genomes \\
                          --docas-dir ./data/docas \\
                          --output ./results \\
                          --experiments te,mse,correlation
"""

import argparse
import json
import sys
from pathlib import Path
from typing import List, Optional

import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
from tqdm import tqdm

from genome_loader import GenomeLoader
from docas_loader import DocasLoader
from emergence_metrics import (
    TransferEntropyAnalyzer,
    MultiScaleEntropyAnalyzer,
    EmergenceDetector
)
from genome_docas_mapper import GenomeDocasMapper


def parse_args() -> argparse.Namespace:
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(
        description="Layer 3 Multi-Scale Emergence Analysis"
    )
    
    parser.add_argument(
        '--data-dir',
        type=Path,
        help='Root directory containing genome/ and docas/ subdirectories'
    )
    
    parser.add_argument(
        '--genome-dir',
        type=Path,
        help='Directory containing genome_{tick}.bin files'
    )
    
    parser.add_argument(
        '--docas-dir',
        type=Path,
        help='Directory containing docas_{tick}.arrow files'
    )
    
    parser.add_argument(
        '--output',
        '-o',
        type=Path,
        default=Path('./layer3_results'),
        help='Output directory for results'
    )
    
    parser.add_argument(
        '--experiments',
        type=str,
        default='all',
        help='Comma-separated list of experiments: te,mse,correlation,prediction,all'
    )
    
    parser.add_argument(
        '--ticks',
        type=str,
        help='Range of ticks to analyze (e.g., "0,1000,100" for start,end,step)'
    )
    
    parser.add_argument(
        '--sample-agents',
        type=int,
        help='Number of agents to sample (for large populations)'
    )
    
    parser.add_argument(
        '--config',
        type=Path,
        help='Path to YAML configuration file'
    )
    
    parser.add_argument(
        '--format',
        choices=['json', 'csv', 'both'],
        default='both',
        help='Output format'
    )
    
    parser.add_argument(
        '--verbose',
        '-v',
        action='store_true',
        help='Enable verbose output'
    )
    
    return parser.parse_args()


def setup_directories(args) -> tuple:
    """
    Setup and validate input directories.
    
    Returns:
        Tuple of (genome_dir, docas_dir)
    """
    if args.data_dir:
        genome_dir = args.data_dir / 'genomes'
        docas_dir = args.data_dir / 'docas'
    else:
        genome_dir = args.genome_dir
        docas_dir = args.docas_dir
    
    if not genome_dir or not docas_dir:
        print("Error: Must specify --data-dir or both --genome-dir and --docas-dir")
        sys.exit(1)
    
    if not genome_dir.exists():
        print(f"Error: Genome directory not found: {genome_dir}")
        sys.exit(1)
    
    if not docas_dir.exists():
        print(f"Error: DOCAS directory not found: {docas_dir}")
        sys.exit(1)
    
    return genome_dir, docas_dir


def load_data(
    genome_dir: Path,
    docas_dir: Path,
    ticks: Optional[List[int]] = None,
    sample_agents: Optional[int] = None,
    verbose: bool = False
) -> tuple:
    """
    Load genome and DOCAS data.
    
    Returns:
        Tuple of (genome_df, docas_df)
    """
    genome_loader = GenomeLoader(genome_dir)
    docas_loader = DocasLoader(docas_dir)
    
    if ticks is None:
        # Use available ticks
        genome_ticks = genome_loader.get_available_ticks()
        docas_ticks = docas_loader.get_available_ticks()
        ticks = sorted(set(genome_ticks) & set(docas_ticks))
    
    if verbose:
        print(f"Loading data for {len(ticks)} ticks...")
    
    # Load time series
    if sample_agents:
        # Sample agents from first tick
        first_genome = genome_loader.load_tick(ticks[0])
        all_agents = first_genome.index.tolist()
        sampled_agents = np.random.choice(
            all_agents,
            size=min(sample_agents, len(all_agents)),
            replace=False
        ).tolist()
        
        genome_df = genome_loader.load_time_series(ticks, sampled_agents)
        docas_df = docas_loader.load_batch_time_series(sampled_agents, ticks)
    else:
        # Load all agents
        genome_df = genome_loader.load_time_series(ticks)
        docas_df = docas_loader.load_batch_time_series(
            docas_loader.load_tick(ticks[0]).index.tolist(),
            ticks
        )
    
    if verbose:
        print(f"Loaded {len(genome_df)} genome records")
        print(f"Loaded {len(docas_df)} DOCAS records")
    
    return genome_df, docas_df


def run_transfer_entropy_analysis(
    genome_df: pd.DataFrame,
    docas_df: pd.DataFrame,
    output_dir: Path,
    verbose: bool = False
) -> dict:
    """Run transfer entropy analysis."""
    if verbose:
        print("\n" + "="*60)
        print("TRANSFER ENTROPY ANALYSIS")
        print("="*60)
    
    analyzer = TransferEntropyAnalyzer(k=1, bins=10)
    
    # Compute TE matrix
    te_matrix = analyzer.compute_te_matrix(genome_df, docas_df)
    
    # Save results
    te_matrix.to_csv(output_dir / 'transfer_entropy_matrix.csv')
    
    if verbose:
        print(f"\nSignificant TE pathways: {te_matrix['significant'].sum()}")
        print(f"Mean TE: {te_matrix['transfer_entropy'].mean():.4f}")
        
        # Show top pathways
        top = te_matrix[te_matrix['significant']].nlargest(5, 'transfer_entropy')
        print("\nTop 5 TE pathways:")
        for idx, row in top.iterrows():
            print(f"  {idx}: TE={row['transfer_entropy']:.4f}, p={row['p_value']:.4f}")
    
    return te_matrix.to_dict()


def run_mse_analysis(
    docas_df: pd.DataFrame,
    output_dir: Path,
    verbose: bool = False
) -> dict:
    """Run multi-scale entropy analysis."""
    if verbose:
        print("\n" + "="*60)
        print("MULTI-SCALE ENTROPY ANALYSIS")
        print("="*60)
    
    analyzer = MultiScaleEntropyAnalyzer(max_scale=20, m=2, r=0.15)
    
    results = {}
    nm_cols = [c for c in docas_df.columns if c not in ['tick']]
    
    for nm in tqdm(nm_cols, desc="Analyzing neuromodulators"):
        result = analyzer.analyze_population(docas_df, nm)
        results[nm] = result
        
        if verbose:
            print(f"\n{nm.upper()}:")
            print(f"  Mean AUC: {result['mean_auc']:.3f} ± {result['std_auc']:.3f}")
    
    # Save results
    with open(output_dir / 'mse_results.json', 'w') as f:
        json.dump(results, f, indent=2, default=str)
    
    # Plot MSE profiles
    fig, axes = plt.subplots(1, len(nm_cols), figsize=(5*len(nm_cols), 4))
    if len(nm_cols) == 1:
        axes = [axes]
    
    for ax, nm in zip(axes, nm_cols):
        result = results[nm]
        scales = list(result['mean_profile'].keys())
        values = list(result['mean_profile'].values())
        stds = list(result['std_profile'].values())
        
        ax.plot(scales, values, 'b-', linewidth=2)
        ax.fill_between(scales, 
                        np.array(values) - np.array(stds),
                        np.array(values) + np.array(stds),
                        alpha=0.3)
        ax.set_xlabel('Scale')
        ax.set_ylabel('Sample Entropy')
        ax.set_title(f'{nm.capitalize()} MSE')
        ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig(output_dir / 'mse_profiles.png', dpi=300)
    plt.close()
    
    return results


def run_correlation_analysis(
    genome_df: pd.DataFrame,
    docas_df: pd.DataFrame,
    output_dir: Path,
    verbose: bool = False
) -> dict:
    """Run genome-DOCAS correlation analysis."""
    if verbose:
        print("\n" + "="*60)
        print("GENOME-DOCAS CORRELATION ANALYSIS")
        print("="*60)
    
    mapper = GenomeDocasMapper()
    
    # Average across time for correlation
    genome_mean = genome_df.groupby(level=0).mean()
    docas_mean = docas_df.groupby(level=0).mean()
    
    # Compute correlation matrix
    corr_matrix = mapper.compute_correlation_matrix(genome_mean, docas_mean)
    
    # Find significant associations
    sig_associations = mapper.find_significant_associations()
    
    # Validate known pathways
    pathway_validation = mapper.validate_known_pathways()
    
    # Build predictive models
    nm_cols = [c for c in docas_mean.columns if c not in ['tick']]
    predictions = {}
    
    for nm in nm_cols:
        model_result = mapper.build_predictive_model(
            genome_mean, docas_mean, nm, 'ridge'
        )
        predictions[nm] = model_result
    
    # Save results
    corr_matrix.to_csv(output_dir / 'correlation_matrix.csv')
    sig_associations.to_csv(output_dir / 'significant_associations.csv', index=False)
    pathway_validation.to_csv(output_dir / 'pathway_validation.csv', index=False)
    
    with open(output_dir / 'predictive_models.json', 'w') as f:
        json.dump(predictions, f, indent=2, default=str)
    
    # Visualize
    mapper.visualize_correlations(output_dir / 'correlation_heatmap.png')
    
    if verbose:
        print(mapper.generate_report())
        
        print("\nPREDICTIVE MODEL PERFORMANCE:")
        for nm, result in predictions.items():
            print(f"  {nm}: R²={result['r2_score']:.3f}, CV R²={result['cv_r2_mean']:.3f}")
    
    return {
        'correlations': corr_matrix.to_dict(),
        'associations': sig_associations.to_dict(),
        'pathways': pathway_validation.to_dict(),
        'predictions': predictions
    }


def run_emergence_detection(
    genome_df: pd.DataFrame,
    docas_df: pd.DataFrame,
    output_dir: Path,
    verbose: bool = False
) -> dict:
    """Run composite emergence detection."""
    if verbose:
        print("\n" + "="*60)
        print("COMPOSITE EMERGENCE DETECTION")
        print("="*60)
    
    detector = EmergenceDetector()
    
    report = detector.detect_emergence(genome_df, docas_df)
    
    # Save results
    with open(output_dir / 'emergence_report.json', 'w') as f:
        json.dump(report, f, indent=2, default=str)
    
    if verbose:
        print(f"\nEmergence Score: {report['emergence_score']:.3f}")
        print(f"Emergence Detected: {report['emergence_detected']}")
        print(f"Significant TE Pathways: {report['n_significant_pathways']}")
        print(f"Mean TE: {report['mean_te']:.4f}")
    
    return report


def main():
    """Main entry point."""
    args = parse_args()
    
    # Setup directories
    genome_dir, docas_dir = setup_directories(args)
    
    # Create output directory
    args.output.mkdir(parents=True, exist_ok=True)
    
    if args.verbose:
        print("="*60)
        print("LAYER 3 MULTI-SCALE EMERGENCE ANALYSIS")
        print("="*60)
        print(f"Genome directory: {genome_dir}")
        print(f"DOCAS directory: {docas_dir}")
        print(f"Output directory: {args.output}")
    
    # Parse tick range
    if args.ticks:
        parts = args.ticks.split(',')
        if len(parts) == 3:
            start, end, step = map(int, parts)
            ticks = list(range(start, end, step))
        else:
            ticks = list(map(int, parts))
    else:
        ticks = None
    
    # Load data
    genome_df, docas_df = load_data(
        genome_dir, docas_dir, ticks, args.sample_agents, args.verbose
    )
    
    # Parse experiments
    experiments = args.experiments.split(',')
    run_all = 'all' in experiments
    
    results = {}
    
    # Run selected experiments
    if run_all or 'te' in experiments:
        results['transfer_entropy'] = run_transfer_entropy_analysis(
            genome_df, docas_df, args.output, args.verbose
        )
    
    if run_all or 'mse' in experiments:
        results['multi_scale_entropy'] = run_mse_analysis(
            docas_df, args.output, args.verbose
        )
    
    if run_all or 'correlation' in experiments:
        results['correlation'] = run_correlation_analysis(
            genome_df, docas_df, args.output, args.verbose
        )
    
    if run_all or 'emergence' in experiments:
        results['emergence'] = run_emergence_detection(
            genome_df, docas_df, args.output, args.verbose
        )
    
    # Save combined results
    if args.format in ['json', 'both']:
        with open(args.output / 'analysis_results.json', 'w') as f:
            json.dump(results, f, indent=2, default=str)
    
    if args.verbose:
        print("\n" + "="*60)
        print("ANALYSIS COMPLETE")
        print("="*60)
        print(f"Results saved to: {args.output}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
