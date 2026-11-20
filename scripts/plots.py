#!/usr/bin/env -S uv run --script

# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "numpy>=2.5",
#     "matplotlib>=3.11.1",
#     "polars>=2.1.0",
# ]
# ///
import sys
from dataclasses import dataclass
from functools import cached_property
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
import polars as pl
from matplotlib.animation import FuncAnimation
from numpy.typing import NDArray


@dataclass(frozen=True)
class Data:
    path: Path

    @cached_property
    def stats(self) -> pl.DataFrame:
        year_f64 = pl.col("year") + (pl.col("month") - 1) / 12
        return (
            pl.scan_csv(self.path / "stats.csv")
            .with_columns(
                year_f64.alias("year_f64"),
            )
            .collect()
        )

    @cached_property
    def pyramid_female(self) -> NDArray[np.float64]:
        return np.loadtxt(self.path / "pyramid_female.csv", delimiter=",")

    @cached_property
    def pyramid_male(self) -> NDArray[np.float64]:
        return np.loadtxt(self.path / "pyramid_male.csv", delimiter=",")

    @cached_property
    def care_need(self) -> NDArray[np.float64]:
        return np.loadtxt(self.path / "care_need.csv", delimiter=",")

    @cached_property
    def classes(self) -> NDArray[np.float64]:
        return np.loadtxt(self.path / "class.csv", delimiter=",")

    @cached_property
    def size(self) -> int:
        return len(self.stats)


@dataclass(frozen=True)
class AnimPlot:
    data: Data

    @cached_property
    def fig_axes(self):
        return plt.subplots(2, 3, figsize=(19.2, 9.6))

    @cached_property
    def plot_dir(self) -> Path:
        plot_dir = self.data.path / "plots"
        plot_dir.mkdir(parents=True)
        return plot_dir

    def plot_at(self, i: int, save: bool):
        fig, axes = self.fig_axes
        for ax in axes.flat:
            ax.clear()

        dat = self.data.stats
        years = dat.get_column("year_f64")
        year = dat.get_column("year")[i]
        month = dat.get_column("month")[i]
        fig.suptitle(f"{year}/{month:02}", fontsize=20)

        for series in ("n_married", "n_working", "n_unemployed", "pop_size"):
            (lines,) = axes[0, 0].plot(years, dat[series], label=series)
            axes[0, 0].plot(
                years[i],
                dat.get_column(series)[i],
                "o",
                color=lines.get_color(),
            )
        axes[0, 0].legend()
        axes[0, 0].set_xlabel("year")
        axes[0, 0].set_ylabel("number of persons")

        for series in ("n_married", "n_working", "n_unemployed"):
            norm_series = dat.get_column(series) / dat.get_column("pop_size")
            (lines,) = axes[0, 1].plot(years, norm_series, label=series)
            axes[0, 1].plot(
                years[i],
                norm_series[i],
                "o",
                color=lines.get_color(),
            )
        axes[0, 1].legend()
        axes[0, 1].set_xlabel("year")
        axes[0, 1].set_ylabel("population share")

        npersons = dat.get_column("pop_size")[i]

        men = self.data.pyramid_male
        women = self.data.pyramid_female
        axes[0, 2].stairs(men[i, 2:] / npersons, label="male")
        axes[0, 2].stairs(women[i, 2:] / npersons, label="female")
        axes[0, 2].legend()
        axes[0, 2].set_xlabel("age")

        care_level = self.data.care_need
        axes[1, 1].bar(range(1, 6), care_level[i, 2:] / npersons)
        axes[1, 1].set_xlabel("care need level")
        axes[1, 1].set_ylabel("population share")

        classes = self.data.classes
        axes[1, 2].bar(range(1, 6), classes[i, 2:] / npersons)
        axes[1, 2].set_xlabel("social class")
        if save:
            fig.savefig(self.plot_dir / f"plot_{year}_{month:02}.svg")

    def make_anim(self, save_each_plot):
        fig, _ = self.fig_axes
        ani = FuncAnimation(
            fig,
            lambda i: self.plot_at(i, save_each_plot),
            frames=range(self.data.size),
        )
        ani.save(self.plot_dir / "animation.mp4")


if __name__ == "__main__":
    run_dir = Path(sys.argv[1])
    anim = AnimPlot(data=Data(run_dir))
    anim.make_anim(True)
