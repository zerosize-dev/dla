package shop.codec

private fun check(ok: Boolean, what: String) {
    if (!ok) throw AssertionError(what)
}

private fun referenceChecksum(bytes: ByteArray): Long {
    var acc = 7
    for (b in bytes) acc = acc * 31 + (b.toInt() and 0xff)
    return acc.toLong() and 0xffffffffL
}

private fun orderWith(lines: Int): Order =
    Order("ord-t", "cust-t", List(lines) { OrderLine("SKU-%04d".format(it), 2, 1250L) })

fun main() {
    val small = FramePacker.pack(orderWith(3))
    check(small.size == FramePacker.frameSize(3), "frame size")

    val handle = Native.open(small)
    check(Native.size(handle) == small.size, "native size")
    check(Native.lines(handle) == 3, "header line count")
    check(Native.checksum(handle) == referenceChecksum(small), "checksum agrees with the reference")
    Native.close(handle)
    check(Native.live() == 0, "closed handle is gone")

    check(orderWith(4).totalCents() == 4 * 2 * 1250L, "order total")

    val stats = Stats()
    val result = OrderEncoder(stats).encode(orderWith(10))
    check(result.checksum == referenceChecksum(FramePacker.pack(orderWith(10))), "encoder checksum")
    check(Native.live() == 0, "regular frames are released")
    check(stats.inlineFrames == 1L, "inline counter")
    println("codec tests passed")
}
