package shop.codec

import java.io.File

/** Counters for the driver and the checkpoint line it prints. */
class Stats {
    var inlineFrames = 0L
    var bulkFrames = 0L
    var checksumTotal = 0L

    fun checkpoint(index: Int): String {
        System.gc()
        val rt = Runtime.getRuntime()
        val heapKb = (rt.totalMemory() - rt.freeMemory()) / 1024
        return "i=$index heapKB=$heapKb nativeLive=${Native.live()} nativeKB=${Native.liveBytes() / 1024}" +
            " audit=${Audit.size()} rssKB=${rssKb()}"
    }

    fun summary(): String =
        "done inline=$inlineFrames bulk=$bulkFrames total=$checksumTotal nativeLive=${Native.live()} audit=${Audit.size()}"

    companion object {
        fun rssKb(): Long {
            val status = File("/proc/self/status")
            if (!status.exists()) return -1
            val line = status.readLines().firstOrNull { it.startsWith("VmRSS:") } ?: return -1
            return line.filter { it.isDigit() }.toLong()
        }
    }
}
