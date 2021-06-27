### Purpose

fsp_bins contains pre-generated rebased Fsp fv files.

rust-firmware-tool will use these pre-generated fsp files and build them into firmware.

you can also generated them too.

### Generate rebased FSP Fv files

1. get fsp release from https://github.com/intel/fsp

2. use SplitFspBin.py to split and rebase Fsp.fd

Fsp_T.bin need rebase to LOADED_FSP_T_BASE
Fsp_M.bin need rebase to LOADED_FSP_M_BASE
Fsp_C.bin need rebase to LOADED_FSP_C_BASE

### TODOs

TBD: write build.rs to generate rebased FSP Fv files
