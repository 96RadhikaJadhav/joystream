'use strict';

const mocha = require('mocha');
const expect = require('chai').expect;
const mock_http = require('node-mocks-http');
const stream_buffers = require('stream-buffers');

const ranges = require.main.require('lib/util/ranges');

describe('ranges', function()
{
  describe('parse()', function()
  {
    it('should parse a full range', function()
    {
      // Range with unit
      var range = ranges.parse('bytes=0-100');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-100');
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(100);

      // Range without unit
      var range = ranges.parse('0-100');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-100');
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(100);

      // Range with custom unit
      //
      var range = ranges.parse('foo=0-100');
      expect(range.unit).to.equal('foo');
      expect(range.range_str).to.equal('0-100');
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(100);
    });

    it('should error out on malformed strings', function()
    {
      expect(() => ranges.parse('foo')).to.throw();
      expect(() => ranges.parse('foo=bar')).to.throw();
      expect(() => ranges.parse('foo=100')).to.throw();
      expect(() => ranges.parse('foo=100-0')).to.throw();
    });

    it('should parse a range without end', function()
    {
      var range = ranges.parse('0-');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-');
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.be.undefined;
    });

    it('should parse a range without start', function()
    {
      var range = ranges.parse('-100');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('-100');
      expect(range.ranges[0][0]).to.be.undefined;
      expect(range.ranges[0][1]).to.equal(100);
    });

    it('should parse multiple ranges', function()
    {
      var range = ranges.parse('0-10,30-40,60-80');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-10,30-40,60-80');
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(10);
      expect(range.ranges[1][0]).to.equal(30);
      expect(range.ranges[1][1]).to.equal(40);
      expect(range.ranges[2][0]).to.equal(60);
      expect(range.ranges[2][1]).to.equal(80);
    });

    it('should merge overlapping ranges', function()
    {
      // Two overlapping ranges
      var range = ranges.parse('0-20,10-30');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-20,10-30');
      expect(range.ranges).to.have.lengthOf(1);
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(30);

      // Three overlapping ranges
      var range = ranges.parse('0-15,10-25,20-30');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-15,10-25,20-30');
      expect(range.ranges).to.have.lengthOf(1);
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(30);

      // Three overlapping ranges, reverse order
      var range = ranges.parse('20-30,10-25,0-15');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('20-30,10-25,0-15');
      expect(range.ranges).to.have.lengthOf(1);
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(30);

      // Adjacent ranges
      var range = ranges.parse('0-10,11-20');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('0-10,11-20');
      expect(range.ranges).to.have.lengthOf(1);
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(20);
    });

    it('should sort ranges', function()
    {
      var range = ranges.parse('10-30,0-5');
      expect(range.unit).to.equal('bytes');
      expect(range.range_str).to.equal('10-30,0-5');
      expect(range.ranges).to.have.lengthOf(2);
      expect(range.ranges[0][0]).to.equal(0);
      expect(range.ranges[0][1]).to.equal(5);
      expect(range.ranges[1][0]).to.equal(10);
      expect(range.ranges[1][1]).to.equal(30);
    });
  });

  describe('send()', function()
  {
    it('should send full files on request', function(done)
    {
      var res = mock_http.createResponse({});
      var in_stream = new stream_buffers.ReadableStreamBuffer({});

      // End-of-stream callback
      var opts = {
        name: 'test.file',
        type: 'application/test',
      };
      ranges.send(res, in_stream, opts, function(err) {
        expect(err).to.not.exist;

        // HTTP handling
        expect(res.statusCode).to.equal(200);
        expect(res.getHeader('content-type')).to.equal('application/test');
        expect(res.getHeader('content-disposition')).to.equal('inline');

        // Data/stream handling
        expect(res._isEndCalled()).to.be.true;
        expect(res._getBuffer().toString()).to.equal('Hello, world!');

        // Notify mocha that we're done.
        done();
      });

      // Simulate file stream
      in_stream.emit('open');
      in_stream.put('Hello, world!');
      in_stream.stop();
    });

    it('should send a range spanning the entire file on request', function(done)
    {
      var res = mock_http.createResponse({});
      var in_stream = new stream_buffers.ReadableStreamBuffer({});

      // End-of-stream callback
      var opts = {
        name: 'test.file',
        type: 'application/test',
        ranges: {
          ranges: [[0, 12]],
        }
      };
      ranges.send(res, in_stream, opts, function(err) {
        expect(err).to.not.exist;

        // HTTP handling
        expect(res.statusCode).to.equal(206);
        expect(res.getHeader('content-type')).to.equal('application/test');
        expect(res.getHeader('content-disposition')).to.equal('inline');
        expect(res.getHeader('content-range')).to.equal('bytes 0-12/*');
        expect(res.getHeader('content-length')).to.equal('13');

        // Data/stream handling
        expect(res._isEndCalled()).to.be.true;
        expect(res._getBuffer().toString()).to.equal('Hello, world!');

        // Notify mocha that we're done.
        done();
      });

      // Simulate file stream
      in_stream.emit('open');
      in_stream.put('Hello, world!');
      in_stream.stop();

    });

    it('should send a small range on request', function(done)
    {
      var res = mock_http.createResponse({});
      var in_stream = new stream_buffers.ReadableStreamBuffer({});

      // End-of-stream callback
      var opts = {
        name: 'test.file',
        type: 'application/test',
        ranges: {
          ranges: [[1, 11]], // Cut off first and last letter
        }
      };
      ranges.send(res, in_stream, opts, function(err) {
        expect(err).to.not.exist;

        // HTTP handling
        expect(res.statusCode).to.equal(206);
        expect(res.getHeader('content-type')).to.equal('application/test');
        expect(res.getHeader('content-disposition')).to.equal('inline');
        expect(res.getHeader('content-range')).to.equal('bytes 1-11/*');
        expect(res.getHeader('content-length')).to.equal('11');

        // Data/stream handling
        expect(res._isEndCalled()).to.be.true;
        expect(res._getBuffer().toString()).to.equal('ello, world');

        // Notify mocha that we're done.
        done();
      });

      // Simulate file stream
      in_stream.emit('open');
      in_stream.put('Hello, world!');
      in_stream.stop();
    });

    it('should send ranges crossing buffer boundaries', function(done)
    {
      var res = mock_http.createResponse({});
      var in_stream = new stream_buffers.ReadableStreamBuffer({
        chunkSize: 3, // Setting a chunk size smaller than the range should
                      // not impact the test.
      });

      // End-of-stream callback
      var opts = {
        name: 'test.file',
        type: 'application/test',
        ranges: {
          ranges: [[1, 11]], // Cut off first and last letter
        }
      };
      ranges.send(res, in_stream, opts, function(err) {
        expect(err).to.not.exist;

        // HTTP handling
        expect(res.statusCode).to.equal(206);
        expect(res.getHeader('content-type')).to.equal('application/test');
        expect(res.getHeader('content-disposition')).to.equal('inline');
        expect(res.getHeader('content-range')).to.equal('bytes 1-11/*');
        expect(res.getHeader('content-length')).to.equal('11');

        // Data/stream handling
        expect(res._isEndCalled()).to.be.true;
        expect(res._getBuffer().toString()).to.equal('ello, world');

        // Notify mocha that we're done.
        done();
      });

      // Simulate file stream
      in_stream.emit('open');
      in_stream.put('Hello, world!');
      in_stream.stop();
    });

    it('should send multiple ranges', function(done)
    {
      var res = mock_http.createResponse({});
      var in_stream = new stream_buffers.ReadableStreamBuffer({});

      // End-of-stream callback
      var opts = {
        name: 'test.file',
        type: 'application/test',
        ranges: {
          ranges: [[1, 3], [5, 7]], // Slice two ranges out
        }
      };
      ranges.send(res, in_stream, opts, function(err) {
        expect(err).to.not.exist;

        // HTTP handling
        expect(res.statusCode).to.equal(206);
        expect(res.getHeader('content-type')).to.satisfy((str) => str.startsWith('multipart/byteranges'));
        expect(res.getHeader('content-disposition')).to.equal('inline');

        // Data/stream handling
        expect(res._isEndCalled()).to.be.true;

        // The buffer should contain both ranges, but with all the That would be
        // "ell" and ", w".
        // It's pretty elaborate having to parse the entire multipart response
        // body, so we'll restrict ourselves to finding lines within it.
        var body = res._getBuffer().toString();
        expect(body).to.contain('\r\nContent-Range: bytes 1-3/*\r\n');
        expect(body).to.contain('\r\nell\r\n');
        expect(body).to.contain('\r\nContent-Range: bytes 5-7/*\r\n');
        expect(body).to.contain('\r\n, w');

        // Notify mocha that we're done.
        done();
      });

      // Simulate file stream
      in_stream.emit('open');
      in_stream.put('Hello, world!');
      in_stream.stop();
    });
  });
});
