from typing import Any, Dict, List, Tuple

import pygal
from pygal.style import CleanStyle


def ia_coverage_histogram(rows: List[Tuple[int, bool, int]]) -> pygal.Graph:
    """
    Note: this returns a raw pygal chart; it does not render it to SVG/PNG

    Rows are tuples of: (year: float or int, in_ia: bool, count: int)
    """

    raw_years = [int(r[0]) for r in rows]
    years_dict = dict()
    if raw_years:
        for y in range(min(raw_years), max(raw_years) + 1):
            years_dict[int(y)] = dict(year=int(y), available=0, missing=0)
        for r in rows:
            if r[1]:
                years_dict[int(r[0])]["available"] = r[2]
            else:
                years_dict[int(r[0])]["missing"] = r[2]

    years: List[Dict[str, Any]] = sorted(years_dict.values(), key=lambda x: x["year"])

    CleanStyle.colors = ("green", "purple")
    label_count = len(years)
    if len(years) > 20:
        label_count = 10
    chart = pygal.StackedBar(
        dynamic_print_values=True,
        style=CleanStyle,
        width=1000,
        height=500,
        x_labels_major_count=label_count,
        show_minor_x_labels=False,
    )
    # chart.title = "Perpetual Access Coverage"
    chart.x_title = "Year"
    # chart.y_title = "Releases"
    chart.x_labels = [str(v["year"]) for v in years]
    chart.add("via Fatcat", [v["available"] for v in years])
    chart.add("Missing", [v["missing"] for v in years])
    return chart


def preservation_by_year_histogram(
    rows: List[Dict], merge_shadows: bool = False
) -> pygal.Graph:
    """
    Note: this returns a raw pygal chart; it does not render it to SVG/PNG

    Rows are dict with keys as preservation types and values as counts (int).
    There is also a 'year' key with float/int value.
    """

    years = sorted(rows, key=lambda x: x["year"])

    if merge_shadows:
        CleanStyle.colors = ("red", "darkolivegreen", "limegreen")
    else:
        CleanStyle.colors = ("red", "darkred", "darkolivegreen", "limegreen")
    label_count = len(years)
    if len(years) > 30:
        label_count = 10
    chart = pygal.StackedBar(
        dynamic_print_values=True,
        style=CleanStyle,
        width=1000,
        height=500,
        x_labels_major_count=label_count,
        show_minor_x_labels=False,
        x_label_rotation=20,
    )
    # chart.title = "Preservation by Year"
    chart.x_title = "Year"
    # chart.y_title = "Count"
    chart.x_labels = [str(y["year"]) for y in years]
    if merge_shadows:
        chart.add("None", [y["none"] + y["shadows_only"] for y in years])
    else:
        chart.add("None", [y["none"] for y in years])
        chart.add("Shadow", [y["shadows_only"] for y in years])
    chart.add("Dark", [y["dark"] for y in years])
    chart.add("Bright", [y["bright"] for y in years])
    return chart


def preservation_by_date_histogram(
    rows: List[Dict], merge_shadows: bool = False
) -> pygal.Graph:
    """
    Note: this returns a raw pygal chart; it does not render it to SVG/PNG

    Rows are dict with keys as preservation types and values as counts (int).
    There is also a 'date' key with str value.
    """

    dates = sorted(rows, key=lambda x: x["date"])

    if merge_shadows:
        CleanStyle.colors = ("red", "darkolivegreen", "limegreen")
    else:
        CleanStyle.colors = ("red", "darkred", "darkolivegreen", "limegreen")
    label_count = len(dates)
    if len(dates) > 30:
        label_count = 10
    chart = pygal.StackedBar(
        dynamic_print_values=True,
        style=CleanStyle,
        width=1000,
        height=500,
        x_labels_major_count=label_count,
        show_minor_x_labels=False,
        x_label_rotation=20,
    )
    # chart.title = "Preservation by Date"
    chart.x_title = "Date"
    # chart.y_title = "Count"
    chart.x_labels = [str(y["date"]) for y in dates]
    if merge_shadows:
        chart.add("None", [y["none"] + y["shadows_only"] for y in dates])
    else:
        chart.add("None", [y["none"] for y in dates])
        chart.add("Shadow", [y["shadows_only"] for y in dates])
    chart.add("Dark", [y["dark"] for y in dates])
    chart.add("Bright", [y["bright"] for y in dates])
    return chart


def preservation_by_volume_histogram(
    rows: List[Dict], merge_shadows: bool = False
) -> pygal.Graph:
    """
    Note: this returns a raw pygal chart; it does not render it to SVG/PNG

    Rows are dict with keys as preservation types and values as counts (int).
    There is also a 'volume' key with str value.
    """

    volumes = sorted(rows, key=lambda x: x["volume"])

    if merge_shadows:
        CleanStyle.colors = ("red", "darkolivegreen", "limegreen")
    else:
        CleanStyle.colors = ("red", "darkred", "darkolivegreen", "limegreen")
    label_count = len(volumes)
    if len(volumes) >= 30:
        label_count = 10
    chart = pygal.StackedBar(
        dynamic_print_values=True,
        style=CleanStyle,
        width=1000,
        height=500,
        x_labels_major_count=label_count,
        show_minor_x_labels=False,
        x_label_rotation=20,
    )
    # chart.title = "Preservation by Volume"
    chart.x_title = "Volume"
    # chart.y_title = "Count"
    chart.x_labels = [str(y["volume"]) for y in volumes]
    if merge_shadows:
        chart.add("None", [y["none"] + y["shadows_only"] for y in volumes])
    else:
        chart.add("None", [y["none"] for y in volumes])
        chart.add("Shadow", [y["shadows_only"] for y in volumes])
    chart.add("Dark", [y["dark"] for y in volumes])
    chart.add("Bright", [y["bright"] for y in volumes])
    return chart
