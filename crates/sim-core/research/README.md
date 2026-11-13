# Layer 3 Analysis

<!-- DOC-002: Stable -->
<!-- DOC-007: Active -->
<!-- Layer 3 Multi-Scale Emergence Analysis -->
<!-- Last Updated: 2026-03-05 -->

Python toolkit for analyzing multi-scale emergence patterns in the Agent Society Simulator.

---

## Overview

This package provides analysis tools for:
- Transfer entropy calculations
- Multi-scale entropy analysis
- Genome-DOCAS data mapping
- Emergence metrics computation

---

## Installation

```bash
cd scripts/layer3_analysis
pip install -r requirements.txt
```

---

## Usage

```bash
# Run main analysis pipeline
python run_analysis.py

# Load and analyze genome data
python genome_loader.py

# Compute emergence metrics
python emergence_metrics.py
```

---

## Modules

| Module | Purpose |
|--------|---------|
| `run_analysis.py` | Main analysis pipeline |
| `genome_loader.py` | Genome data loading |
| `docas_loader.py` | DOCAS data loading |
| `emergence_metrics.py` | Transfer entropy, multi-scale entropy |
| `genome_docas_mapper.py` | Data mapping utilities |

---

*Last Updated: 2026-03-05*
