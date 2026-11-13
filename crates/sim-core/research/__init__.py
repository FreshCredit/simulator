"""
Layer 3 Analysis: Multi-Scale Emergence Detection

Analyzes cross-scale patterns from Layer 3 (Biology/Genome) to Layer 4 (Neurobiology/DOCAS).

Usage:
    python -m layer3_analysis.run_analysis --simulation-dir ./data/sims --output ./results
"""

__version__ = "0.1.0"
__author__ = "Agent Society Simulator Research Team"

from .genome_loader import GenomeLoader
from .docas_loader import DocasLoader
from .emergence_metrics import TransferEntropyAnalyzer, MultiScaleEntropyAnalyzer
from .genome_docas_mapper import GenomeDocasMapper

__all__ = [
    "GenomeLoader",
    "DocasLoader", 
    "TransferEntropyAnalyzer",
    "MultiScaleEntropyAnalyzer",
    "GenomeDocasMapper",
]
