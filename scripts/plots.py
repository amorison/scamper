# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "numpy>=2.5",
#     "matplotlib>=3.11.1",
#     "pandas>=3.0.5",
# ]
# ///
import sys
from dataclasses import dataclass
from functools import cached_property
from pathlib import Path

import matplotlib.pyplot as plt
import pandas as pd
from matplotlib.animation import FuncAnimation


@dataclass(frozen=True)
class Data:
    path: Path

    @cached_property
    def stats(self) -> pd.DataFrame:
        return pd.read_csv(self.path / "stats.csv")

    @cached_property
    def pyramid_female(self) -> pd.DataFrame:
        return pd.read_csv(self.path / "pyramid_female.csv")

    @cached_property
    def pyramid_male(self) -> pd.DataFrame:
        return pd.read_csv(self.path / "pyramid_male.csv")

    @cached_property
    def care_need(self) -> pd.DataFrame:
        return pd.read_csv(self.path / "care_need.csv")

    @cached_property
    def classes(self) -> pd.DataFrame:
        return pd.read_csv(self.path / "class.csv")

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
        years = dat["year"] + (dat["month"] - 1) / 12
        year = dat["year"].iloc[i]
        month = dat["month"].iloc[i]
        fig.suptitle(f"{year}/{month:02}", fontsize=20)

        for series in ("n_married", "n_working", "n_unemployed", "pop_size"):
            (lines,) = axes[0, 0].plot(years, dat[series], label=series)
            axes[0, 0].plot(
                years.iloc[i],
                dat[series].iloc[i],
                "o",
                color=lines.get_color(),
            )
        axes[0, 0].legend()
        axes[0, 0].set_xlabel("year")
        axes[0, 0].set_ylabel("number of persons")

        for series in ("n_married", "n_working", "n_unemployed"):
            norm_series = dat[series] / dat["pop_size"]
            (lines,) = axes[0, 1].plot(years, norm_series, label=series)
            axes[0, 1].plot(
                years.iloc[i],
                norm_series.iloc[i],
                "o",
                color=lines.get_color(),
            )
        axes[0, 1].legend()
        axes[0, 1].set_xlabel("year")
        axes[0, 1].set_ylabel("population share")

        npersons = dat["pop_size"].iloc[i]

        men = self.data.pyramid_male
        women = self.data.pyramid_female
        axes[0, 2].stairs(men.iloc[i, 2:] / npersons, label="male")
        axes[0, 2].stairs(women.iloc[i, 2:] / npersons, label="female")
        axes[0, 2].legend()
        axes[0, 2].set_xlabel("age")

        care_level = self.data.care_need
        axes[1, 1].bar(range(1, 6), care_level.iloc[i, 2:] / npersons)
        axes[1, 1].set_xlabel("care need level")
        axes[1, 1].set_ylabel("population share")

        classes = self.data.classes
        axes[1, 2].bar(range(1, 6), classes.iloc[i, 2:] / npersons)
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
