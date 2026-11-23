"""Native handler and real loopback HTTP acceptance checks; run from any directory."""
import json
import os
import http.client
import select
import socket
from pathlib import Path
import subprocess
import tempfile
import unittest
from contextlib import closing

ROOT = Path(__file__).resolve().parents[2]
COMPILER = Path(os.environ.get("LOBO_COMPILER", ROOT / "target/release/lobo"))
BUILD = Path(os.environ.get("LOBO_WEB_EXAMPLE_DIR", ROOT / "dist/web-server"))
SERVER = BUILD / "server"
HANDLER = BUILD / "handler"


class HandlerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(prefix="lobo-web-test-")
        cls.handlers = []
        for backend in ["c", "llvm"]:
            handler = Path(cls.temp.name) / backend
            subprocess.run([str(COMPILER), "compile", str(ROOT / "examples/web-server/app.lobo"), "--source", str(ROOT / "libraries/http/http.lobo"), "--backend", backend, "-o", str(handler)], check=True, capture_output=True)
            cls.handlers.append(handler)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_native_lobo_routes(self):
        cases = [
            ("GET", "/", 200, "Hello from native Lobo!\n"),
            ("GET", "/health", 200, '{"status":"ok","language":"lobo"}\n'),
            ("GET", "/greet/é", 200, "Hello, é!\n"),
            ("GET", "/missing", 404, "Not found\n"),
            ("POST", "/", 405, "Method not allowed\n"),
            ("HEAD", "/", 200, "Hello from native Lobo!\n"),
        ]
        name="a"*(2048-len("/greet/"))
        cases.append(("GET","/greet/"+name,200,f"Hello, {name}!\n"))
        for handler in self.handlers:
            for method, path, status, body in cases:
                with self.subTest(backend=handler.name,method=method,path=path):
                    result=subprocess.run([str(handler),"--entry","example.web.handle","--arg",f"Text:{method.encode().hex()}","--arg",f"Text:{path.encode().hex()}","--fuel","10000","--heap-cells","10000"],check=True,capture_output=True,text=True)
                    fields=json.loads(result.stdout)["value"]["value"]["fields"]
                    self.assertEqual(fields["status"]["value"],str(status))
                    self.assertEqual(fields["body"]["value"],body)


class HttpTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.process = subprocess.Popen([str(SERVER), "--listen", "127.0.0.1:0", "--handler", str(HANDLER)], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        if not select.select([cls.process.stdout],[],[],10)[0]:
            cls.process.kill()
            raise AssertionError("server did not announce readiness")
        line = cls.process.stdout.readline()
        if not line:
            raise AssertionError(cls.process.stderr.read())
        cls.port = int(json.loads(line)["address"].rsplit(":",1)[1])

    @classmethod
    def tearDownClass(cls):
        cls.process.terminate()
        cls.process.communicate(timeout=5)

    def request(self,method,path):
        with closing(http.client.HTTPConnection("127.0.0.1",self.port,timeout=5)) as connection:
            connection.request(method,path)
            response=connection.getresponse()
            return response.status, dict(response.getheaders()), response.read()

    def raw(self,request):
        with socket.create_connection(("127.0.0.1",self.port),timeout=5) as connection:
            connection.sendall(request)
            response=http.client.HTTPResponse(connection)
            response.begin()
            return response.status,response.read()

    def test_routes_utf8_byte_lengths_head_and_method_policy(self):
        for method,path,status,body in [
            ("GET","/",200,b"Hello from native Lobo!\n"),
            ("GET","/health",200,b'{"status":"ok","language":"lobo"}\n'),
            ("GET","/greet/%C3%A9?unused=yes",200,"Hello, é!\n".encode()),
            ("GET","/missing",404,b"Not found\n"),
            ("POST","/",405,b"Method not allowed\n"),
            ("HEAD","/",200,b""),
        ]:
            with self.subTest(method=method,path=path):
                actual,headers,content=self.request(method,path)
                self.assertEqual(actual,status)
                self.assertEqual(content,body)
                self.assertEqual(headers["Connection"],"close")
                self.assertIn("Date",headers)
                self.assertEqual(int(headers["Content-Length"]),len(b"Hello from native Lobo!\n") if method=="HEAD" else len(body))
                if status==405: self.assertEqual(headers["Allow"],"GET, HEAD")

    def test_invalid_requests_and_header_deadline(self):
        for wire,status in [
            (b"GET / HTTP/1.1\r\n\r\n",400),
            (b"GET / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n",400),
            (b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 1\r\n\r\nx",413),
            (b"GET /"+b"a"*2048+b" HTTP/1.1\r\nHost: a\r\n\r\n",414),
            (b"GET / HTTP/1.1\r\nHost: a\r\nX-Pad: "+b"a"*8192+b"\r\n\r\n",431),
            (b"GET /greet/%0a HTTP/1.1\r\nHost: a\r\n\r\n",400),
        ]:
            with self.subTest(status=status): self.assertEqual(self.raw(wire)[0],status)
        with socket.create_connection(("127.0.0.1",self.port),timeout=5) as connection:
            connection.sendall(b"GET / HTTP/1.1\r\nHost:")
            response=http.client.HTTPResponse(connection)
            response.begin()
            self.assertEqual(response.status,408)
            response.read()

    def test_non_loopback_listener_is_rejected(self):
        result=subprocess.run([str(SERVER),"--listen","0.0.0.0:0","--handler",str(HANDLER)],capture_output=True,text=True,timeout=5)
        self.assertNotEqual(result.returncode,0)
        self.assertIn("loopback",result.stderr)

    def test_largest_accepted_greeting_fits_the_handler_budgets(self):
        name="a"*(2048-len("/greet/"))
        status,headers,body=self.request("GET","/greet/"+name)
        self.assertEqual(status,200)
        self.assertEqual(body,f"Hello, {name}!\n".encode())

    def test_handler_start_failure_becomes_a_generic_http_error(self):
        with tempfile.TemporaryDirectory(prefix="lobo-web-invalid-handler-") as directory:
            handler=Path(directory)/"not-an-executable"
            handler.write_text("not executable")
            process=subprocess.Popen([str(SERVER),"--listen","127.0.0.1:0","--handler",str(handler),"--max-requests","1"],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
            try:
                self.assertTrue(select.select([process.stdout],[],[],10)[0])
                port=int(json.loads(process.stdout.readline())["address"].rsplit(":",1)[1])
                with closing(http.client.HTTPConnection("127.0.0.1",port,timeout=5)) as connection:
                    connection.request("GET","/")
                    response=connection.getresponse()
                    self.assertEqual(response.status,500)
                    self.assertEqual(response.read(),b"Handler failed\n")
                process.wait(timeout=5)
                self.assertEqual(process.returncode,0)
            finally:
                if process.poll() is None: process.terminate()
                process.communicate(timeout=5)


if __name__ == "__main__":
    unittest.main()
