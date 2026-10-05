# Candidate OCDraw conformance

This unpublished candidate develops the standalone OCDraw 0.1.0 contract.
`ocdraw/cases.json` lists every fixture, its expected validity and, for invalid
cases, the required diagnostic. The production reader exercises this index.

The collection covers empty and optional stream sets, placed geometry, scope
ownership and order for model/paper/shared blocks, transformed bounds, and
view/clip/workspace references. Obsolete owner columns, order streams and
stream directories are invalid. Entity schemas start at v1 within OCDraw.

Released numbered collections remain immutable historical IFCCAD material.

The independent [IFCX-CAD candidate collection](ifcx-native-cad/README.md)
exercises its provisional viewport profile with strict production native
readback. It is separate from the OCDraw cases in this directory and does not
modify any numbered collection.
Their package readers and schemas are available in Git history; they are not
interpreted by the current standalone reader.
