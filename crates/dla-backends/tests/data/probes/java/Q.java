import java.io.*; import java.sql.*;
public class Q {
  void a() throws IOException { InputStream in = new FileInputStream("a"); in.read(); }
  void b(Connection c) throws SQLException { Statement s = c.createStatement(); ResultSet r = s.executeQuery("x"); r.next(); }
  void c() throws IOException { OutputStream o = new FileOutputStream("b"); o.write(1); o.close(); }
  class Inner { int x; }
  Runnable r = new Runnable(){ public void run(){} };
}
