"""Independent literal conformance CBOR helpers; never call a production codec."""
import struct

def decode(data):
    offset = 0

    def read():
        nonlocal offset
        initial = data[offset]
        offset += 1
        major, extra = divmod(initial, 32)
        if initial in (244, 245):
            return initial == 245
        if initial == 251:
            value = struct.unpack_from('>d', data, offset)[0]
            offset += 8
            return value
        if extra < 24:
            n = extra
        else:
            size = {24: 1, 25: 2, 26: 4, 27: 8}[extra]
            n = int.from_bytes(data[offset:offset+size], 'big')
            offset += size
        if major == 0:
            return n
        if major == 1:
            return -1-n
        if major in (2, 3):
            value = data[offset:offset+n]
            offset += n
            return value.decode('utf-8') if major == 3 else value
        if major == 4:
            return [read() for _ in range(n)]
        if major == 5:
            return {read(): read() for _ in range(n)}
        raise ValueError(initial)

    result = read()
    assert offset == len(data)
    return result


def encode(value):
    def length(major, n):
        if n < 24:
            return bytes([major*32+n])
        for extra, size in [(24, 1), (25, 2), (26, 4), (27, 8)]:
            if n < 1 << (size*8):
                return bytes([major*32+extra])+n.to_bytes(size, 'big')
        raise ValueError(n)
    if value is None:
        return b'\xf6'
    if isinstance(value, bool):
        return b'\xf5' if value else b'\xf4'
    if isinstance(value, int):
        return length(0, value) if value >= 0 else length(1, -1-value)
    if isinstance(value, float):
        return b'\xfb'+struct.pack('>d', value)
    if isinstance(value, str):
        b = value.encode('utf-8')
        return length(3, len(b))+b
    if isinstance(value, bytes):
        return length(2, len(value))+value
    if isinstance(value, list):
        return length(4, len(value))+b''.join(encode(x) for x in value)
    if isinstance(value, dict):
        return length(5, len(value))+b''.join(encode(k)+encode(v) for k, v in value.items())
    raise TypeError(value)

