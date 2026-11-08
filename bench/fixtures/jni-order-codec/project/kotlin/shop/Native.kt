package shop.codec

/** Bindings to the frame codec in libcodec (native/src/lib.rs). */
object Native {
    init {
        System.loadLibrary("codec")
    }

    @JvmStatic external fun open(data: ByteArray): Long
    @JvmStatic external fun checksum(handle: Long): Long
    @JvmStatic external fun size(handle: Long): Int
    @JvmStatic external fun lines(handle: Long): Int
    @JvmStatic external fun close(handle: Long)
    @JvmStatic external fun live(): Int
    @JvmStatic external fun liveBytes(): Long
}
