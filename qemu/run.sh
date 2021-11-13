#!/bin/sh

#ISO="winnt40wks_sp1_en.iso"
#ISO="./Microsoft Visual C++ 4.0a RISC Edition for MIPS (ISO)/VCPP-4.00-RISC-MIPS.iso"

qemu-system-mips64el \
    -M magnum \
    -cpu VR5432 \
    -m 128 \
    -net nic \
    -net user,hostfwd=tcp::5555-:42069 \
    -global ds1225y.filename=nvram \
    -global ds1225y.size=8200 \
    -L . \
    -hda nt4.qcow2 \
    -cdrom "$ISO"

