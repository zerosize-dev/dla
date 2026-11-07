import java.io.*; import java.nio.file.*; import java.util.stream.*;
public class P {
  ThreadLocal<byte[]> tl = new ThreadLocal<>();
  class Inner { int x; }
  long n() throws IOException { Stream<String> s = Files.lines(Paths.get("x")); return s.count(); }
  void f() throws IOException { new FileInputStream("a").read(); }
}
