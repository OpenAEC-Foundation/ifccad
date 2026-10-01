# Standalone OCDraw inspection

The application accepts one OCDraw, DXF or DWG file. The default browser worker
calls `ocdraw-browser`; native processing is an explicit user choice via a local
service. Both use the standalone reader and converter. There is no package
entry selection, ZIP reconstruction or IFCX graph conversion.

The UI displays physical/typed drawing information and diagnostics. Geometric
rendering remains later work. CAD export checks actual file readback and version;
this does not assert exhaustive target-codec fidelity. Numeric conversion checks
remain independent hard requirements in both directions.

Worker transport, cancellation, staging confinement, job tokens, size limits and
cleanup remain tested. Hosted deployment is outside this migration task.
