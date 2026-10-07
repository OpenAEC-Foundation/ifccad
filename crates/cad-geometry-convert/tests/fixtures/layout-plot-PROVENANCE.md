# Layout plot boundary qualification

Base opencadcodec: fe69506cb99dea6f4c4a73b690a27fdf04403ea0, with the repository's three explicit viewport development repairs. The repairs do not change layout plot fields. Tests use production DXF and AC1032 DWG readers/writers.

`layout-plot-reference.dxf` is hand-authored. Its inch selector deliberately accompanies millimetre dimensions, margins and offsets. It has a dormant zero standard preset with an active custom 2/1 mapping and layout flag bit 2, independently of plot flags.

`layout_plot_codec` confirms: physical fields retain their numerical millimetre values; custom/standard selectors and numerator/denominator survive; bit 1 and bit 2 survive independently for two layouts in both DXF and DWG. Medium-only constructor-default state remains recognizable after file readback. Pixel selector and dimensions survive numerically, but this does not establish pixel/mm calibration or certify a native raster-medium mapping. Pixel transfers without that mapping must diagnose loss.

The native adapters must assess semantic field values, not replay raw retained plot codes. No measurements or new codec patches were run/introduced for this qualification.
