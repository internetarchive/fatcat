
import pygal
from pygal.style import CleanStyle

def ia_coverage_histogram(rows):
    """
    Note: this returns a raw pygal chart; it does not render it to SVG/PNG
    """

    raw_years = [int(r[0]) for r in rows]
    years = dict()
    for y in range(min(raw_years), max(raw_years)+1):
        years[int(y)] = dict(year=int(y), available=0, missing=0)
    for r in rows:
        if r[1]:
            years[int(r[0])]['available'] = r[2]
        else:
            years[int(r[0])]['missing'] = r[2]

    years = sorted(years.values(), key=lambda x: x['year'])

    CleanStyle.colors = ("green", "purple")
    label_count = len(years)
    if len(years) > 20:
        label_count = 10
    chart = pygal.StackedBar(dynamic_print_values=True, style=CleanStyle,
        width=1000, height=500, x_labels_major_count=label_count,
        show_minor_x_labels=False)
    #chart.title = "Perpetual Access Coverage"
    chart.x_title = "Year"
    #chart.y_title = "Releases"
    chart.x_labels = [str(y['year']) for y in years]
    chart.add('Available', [y['available'] for y in years])
    chart.add('Missing', [y['missing'] for y in years])
    return chart

