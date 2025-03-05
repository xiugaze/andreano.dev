from http.server import SimpleHTTPRequestHandler, HTTPServer

class CORSRequestHandler(SimpleHTTPRequestHandler):

    def end_headers (self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', '*')
        self.send_header('Access-Control-Allow-Headers', '*')
        SimpleHTTPRequestHandler.end_headers(self)
        
    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

httpd = HTTPServer(('localhost', 8000), CORSRequestHandler)
httpd.serve_forever()
