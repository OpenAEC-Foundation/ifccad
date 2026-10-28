# Shared CAD workspace fields

This companion maps identifier-free VPORT/VIEWPORT view, canvas-frame,
grid/snap and UCS-frame values to the core `workspace_kernel`. Both independent
format adapters own names, IDs, references, current-choice qualification,
source residuals and loss policy. No CAD runtime enters either native core.

Preparation checks finite values and exact UCS axes before copying. Disabled
zero snap spacing and stored-UCS activation stay intact. The applying functions
update only their field family on existing CAD records; viewport aids never
replace frame, view, visibility or clipping. Field-specific substitutions and
known file-portability limits return `WorkspaceFieldLoss` evidence. VIEWPORT
GridFlags remain in CadDocument but the pinned file routes omit them; VPORT
flags survive. Reject enforcement belongs to the calling format adapter.

The [workspace contract](../../docs/workspace-state.md) records coordinate
roles, optional choices and transport qualification. Tests use literal DXF and
real DXF/AC1032 DWG readback against the explicitly selected codec base and
viewport-off patch. They are correctness checks, not controlled measurements.
