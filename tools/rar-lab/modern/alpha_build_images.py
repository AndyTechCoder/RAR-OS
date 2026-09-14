"""Private three-image native Alpha framing; unchanged per-service PE limits."""

def unpack(data,inspect):
    import base64
    if type(data) is not bytes or len(data)>6*1024*1024:
        raise ValueError("bounded signed build transfer")
    lines=data.split(b"\n")
    if len(lines)!=8 or lines[-2:]!=[b"RAR-SIGNED-BUILD:END",b""]:
        raise ValueError("signed build transfer framing")
    result={}
    for i,name in enumerate(("modern.efi","modern-service.efi","modern-network.efi")):
        if lines[i*2]!=("RAR-SIGNED-FILE:"+name).encode():
            raise ValueError("signed artifact order")
        value=base64.b64decode(lines[i*2+1],validate=True)
        if base64.b64encode(value)!=lines[i*2+1]:
            raise ValueError("canonical signed artifact")
        inspect(value,name!="modern.efi")
        result[name]=value
    # Bind the independently inspected fixed network PE to the exact kernel.
    if result["modern.efi"].count(result["modern-network.efi"])!=1:
        raise ValueError("exact fixed network service embedded once")
    return result
