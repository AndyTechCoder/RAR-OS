"""Independent fixed FAT16 boot-package check for Modern cloud tooling.
Bytes only. Does not execute a packager, firmware, target or virtual machine.
"""
import base64
import hashlib
import struct

SIZE=16777216
PREFIX=b"RAR-MODERN-BOOT:"

def expected(efi):
    if type(efi) is not bytes or not 1024<=len(efi)<=2097152 or efi[:2]!=b"MZ":
        raise ValueError("bounded previously PE-validated UEFI input")
    image=bytearray(SIZE)
    def u16(at,value): struct.pack_into("<H",image,at,value)
    def u32(at,value): struct.pack_into("<I",image,at,value)
    image[:3]=bytes((0xeb,0x3c,0x90));image[3:11]=b"RAROS   "
    for at,value in ((11,512),(14,1),(17,512),(19,32768),(22,32),(24,32),(26,64)):
        u16(at,value)
    image[13]=4;image[16]=2;image[21]=0xf8;image[36]=0x80;image[38]=0x29
    u32(39,0x52415231);image[43:54]=b"RAR ALPHA  ";image[54:62]=b"FAT16   "
    image[510:512]=bytes((0x55,0xaa))
    count=(len(efi)+2047)//2048
    for fat in (512,33*512):
        for index,value in enumerate((0xfff8,0xffff,0xffff,0xffff)):
            u16(fat+2*index,value)
        for index in range(count):
            u16(fat+2*(index+4),0xffff if index==count-1 else index+5)
    def cluster(n): return 97*512+(n-2)*2048
    def entry(at,name,attributes,first,size):
        image[at:at+11]=name;image[at+11]=attributes
        u16(at+24,((2026-1980)<<9)|(1<<5)|1)
        u16(at+26,first);u32(at+28,size)
    entry(65*512,b"EFI        ",0x10,2,0)
    entry(cluster(2),b".          ",0x10,2,0)
    entry(cluster(2)+32,b"..         ",0x10,0,0)
    entry(cluster(2)+64,b"BOOT       ",0x10,3,0)
    entry(cluster(3),b".          ",0x10,3,0)
    entry(cluster(3)+32,b"..         ",0x10,2,0)
    entry(cluster(3)+64,b"BOOTX64 EFI",0x20,4,len(efi))
    image[cluster(4):cluster(4)+len(efi)]=efi
    return bytes(image)

def validate(image,efi):
    if type(image) is not bytes or len(image)!=SIZE or image!=expected(efi):
        raise ValueError("boot image differs from exact fixed FAT16/UEFI contract")
    return hashlib.sha256(image).hexdigest()

def unpack(transfer,efi):
    if type(transfer) is not bytes or len(transfer)!=len(PREFIX)+4*((SIZE+2)//3)+1:
        raise ValueError("exact bounded boot transfer length")
    if not transfer.startswith(PREFIX) or not transfer.endswith(b"\n"):
        raise ValueError("fixed boot transfer framing")
    encoded=transfer[len(PREFIX):-1]
    try: image=base64.b64decode(encoded,validate=True)
    except ValueError as error: raise ValueError("malformed boot base64") from error
    if base64.b64encode(image)!=encoded:
        raise ValueError("canonical boot base64")
    validate(image,efi)
    return image

def self_test():
    efi=b"MZ"+bytes(1022)
    image=expected(efi)
    assert len(validate(image,efi))==64
    transfer=PREFIX+base64.b64encode(image)+b"\n"
    assert unpack(transfer,efi)==image
    rejected=0
    def reject(fn):
        nonlocal rejected
        try: fn()
        except ValueError: rejected+=1
        else: raise AssertionError("invalid boot package accepted")
    for at in (0,11,13,14,16,17,19,22,36,39,43,54,510,512,33*512,65*512,97*512,101*512,SIZE-1):
        changed=bytearray(image);changed[at]^=1
        reject(lambda changed=bytes(changed):validate(changed,efi))
    for data in (b"",transfer[:-1],transfer+b"x",b"X"+transfer[1:]):
        reject(lambda data=data:unpack(data,efi))
    reject(lambda:validate(image,efi[:-1]+b"x"))
    reject(lambda:expected(b""))
    return rejected

if __name__=="__main__":
    import sys
    if sys.argv!=[sys.argv[0],"--self-test"] or not sys.flags.isolated or not sys.dont_write_bytecode:
        raise SystemExit("pure boot-package self-test only")
    print("Modern boot package:",self_test(),"negative fixtures; no target execution")
