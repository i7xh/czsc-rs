# Weight Backtest Engine (Rust + PyO3)

[![PyPI version](https://img.shields.io/pypi/v/weight_backtest_pyo3)](https://pypi.org/project/weight_backtest_pyo3/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust CI](https://github.com/yourusername/weight_backtest_pyo3/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/yourusername/weight_backtest_pyo3/actions/workflows/rust-ci.yml)

一个高性能的权重回测引擎，使用 Rust 编写并通过 PyO3 提供 Python 绑定。专为量化交易策略的回测设计，支持多标的、多日期的复杂回测场景。

## 功能特点

- 🚀 **高性能**：使用 Rust 实现核心逻辑，比纯 Python 实现快 10-100 倍
- 📊 **全面指标**：计算投资组合指标、标的级指标和交易级指标
- ⚙️ **灵活配置**：支持自定义交易费率、价格精度、权重策略等
- 🧩 **简单集成**：通过 Python 接口轻松集成到现有量化研究框架
- 🧪 **详细结果**：返回多层次回测结果，便于深入分析策略表现
- 🧵 **并行计算**：支持多线程并行处理，加速大规模回测

## 安装

### 前提条件

- Python 3.7+
- Rust 工具链（安装指南：[rustup](https://rustup.rs/))

### 通过 pip 安装

```bash
pip install weight_backtest_pyo3

pip install maturin
maturin develop
```

### python 调用 pyo3 包
```python
import pandas as pd
import polars as pl
from weight_backtest_pyo3 import WeightBacktest

dfw = pd.read_feather("/your_feather_dir/weight_example.feather")
df = pl.from_pandas(dfw)
wbt = WeightBacktest(df, 3, "ts", 0.0002, 252, 1)
r = wbt.run_backtest()
```

