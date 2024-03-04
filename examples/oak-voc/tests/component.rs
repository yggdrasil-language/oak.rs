use oak_awsl::widget_name_from_stem;

#[test]
fn stem_to_widget_name_examples() {
    assert_eq!(widget_name_from_stem("counter"), "counter");
    assert_eq!(widget_name_from_stem("todo_item"), "todo_item");
    assert_eq!(widget_name_from_stem("interactive-col-plot"), "interactive_col_plot");
    assert_eq!(widget_name_from_stem("[slug]"), "slug");
    assert_eq!(widget_name_from_stem("chart-status"), "chart_status");
    assert_eq!(widget_name_from_stem("BenchReport"), "bench_report");
    assert_eq!(widget_name_from_stem("TodoItem"), "todo_item");
    assert_eq!(widget_name_from_stem("CoverageReport"), "coverage_report");
}
