'use strict';

const uuid = require('uuid');
const stream_buf = require('stream-buffers');

const debug = require('debug')('joystream:util:ranges');

/*****************************************************************************
 * Range parsing
 **/

/**
 * Parse a range string, e.g. '0-100' or '-100' or '0-'. Return the values
 * in an array of int or undefined (if not provided).
 **/
function _parse_range(range)
{
  var matches = range.match(/^(\d+-\d+|\d+-|-\d+|\*)$/);
  if (!matches) {
    throw new Error(`Not a valid range: ${range}`);
  }

  var vals = matches[1].split('-').map((v) => {
    return v === '*' || v === '' ? undefined : parseInt(v);
  });

  if (vals[1] <= vals[0]) {
    throw new Error(`Invalid range: start "${vals[0]}" must be before end "${vals[1]}".`);
  }

  return [vals[0], vals[1]];
};


/**
 * Parse a range header value, e.g. unit=ranges, where ranges
 * are a comman separated list of individual ranges, and unit is any
 * custom unit string. If the unit (and equal sign) are not given, assume
 * 'bytes'.
 **/
function parse(range_str)
{
  var res = {};
  debug('Parse range header value:', range_str);
  var matches = range_str.match(/^(([^\s]+)=)?((?:(?:\d+-\d+|-\d+|\d+-),?)+)$/)
  if (!matches) {
    throw new Error(`Not a valid range header: ${range_str}`);
  }

  res.unit = matches[2] || 'bytes';
  res.range_str = matches[3];
  res.ranges = [];

  // Parse individual ranges
  var ranges = []
  res.range_str.split(',').forEach((range) => {
    ranges.push(_parse_range(range));
  });

  // Merge ranges into result.
  ranges.forEach((new_range) => {
    debug('Found range:', new_range);

    var is_merged = false;
    for (var i in res.ranges) {
      var old_range = res.ranges[i];

      // Skip if the new range is fully separate from the old range.
      if (old_range[1] + 1 < new_range[0] || new_range[1] + 1 < old_range[0]) {
        debug('Range does not overlap with', old_range);
        continue;
      }

      // If we know they're adjacent or overlapping, we construct the
      // merged range from the lower start and the higher end of both
      // ranges.
      var merged = [
        Math.min(old_range[0], new_range[0]),
        Math.max(old_range[1], new_range[1])
      ];
      res.ranges[i] = merged;
      is_merged = true;
      debug('Merged', new_range, 'into', old_range, 'as', merged);
    }

    if (!is_merged) {
      debug('Non-overlapping range!');
      res.ranges.push(new_range);
    }
  });

  // Finally, sort ranges
  res.ranges.sort((first, second) => {
    if (first[0] === second[0]) return 0; // Should not happen due to merging.
    return (first[0] < second[0]) ? -1 : 1;
  });

  debug('Result of parse is', res);
  return res;
}


/**
 * Async version of parse().
 **/
function parseAsync(range_str, cb)
{
  try {
    cb(parse(range_str));
  } catch (err) {
    cb(null, err);
  }
}


/*****************************************************************************
 * Range streaming
 **/

/**
 * The class writes parts specified in the options to the response. If no ranges
 * are specified, the entire stream is written. At the end, the given callback
 * is invoked - if an error occurred, it is invoked with an error parameter.
 *
 * Note that the range implementation can be optimized for streams that support
 * seeking.
 **/
class RangeSender
{
  constructor(response, stream, opts, end_callback)
  {
    // Parameters
    this.response = response;
    this.stream = stream;
    this.opts = opts;
    this.end_callback = end_callback;

    // Options
    this.name = opts.name || 'content.bin';
    this.type = opts.type || 'application/octet-stream';
    this.size = opts.size;
    this.ranges = opts.ranges;
    this.download = opts.download || false;

    // Range handling related state.
    this.read_offset = 0;             // Nothing read so far
    this.range_index = -1;            // No range index yet.
    this.range_boundary = undefined;  // Generate boundary when needed.

    // Event handlers
    this.handlers = {};
  }

  on_error(err)
  {
    // Assume hiding the actual error is best, and default to 404.
    debug('Error:', err);
    this.response.status(err.code || 404).send({
      message: err.message || `File not found: ${name}`
    });
    if (this.end_callback) this.end_callback(err);
  }

  on_end()
  {
    debug('End of stream.');
    this.response.end();
    if (this.end_callback) this.end_callback();
  }


  // **** No ranges
  on_open_no_range()
  {
    // File got opened, so we can set headers/status
    debug('Open succeeded:', this.name, this.type);

    this.response.status(200);
    this.response.contentType(this.type);
    this.response.header('Accept-Ranges', 'bytes');
    this.response.header('Content-Transfer-Encoding', 'binary');

    if (this.download) {
      this.response.header('Content-Disposition', `attachment; filename="${this.name}"`);
    }
    else {
      this.response.header('Content-Disposition', 'inline');
    }

    if (this.size) {
      this.response.header('Content-Length', this.size);
    }
  }


  on_data_no_range(chunk)
  {
    // As simple as it can be.
    this.response.write(Buffer.from(chunk, 'binary'));
  }

  // *** With ranges
  next_range_headers()
  {
    // Next range
    this.range_index += 1;
    if (this.range_index >= this.ranges.ranges.length) {
      debug('Cannot advance range index; we are done.');
      return;
    }

    // Calculate this range's size.
    var range = this.ranges.ranges[this.range_index];
    var size = range[1] - range[0] + 1;
      // TODO undefined start/end?

    // Write headers, but since we may be in a multipart situation, write them
    // explicitly to the stream.
    return {
      'Content-Length': `${size}`,
      'Content-Range': `bytes ${range[0]}-${range[1]}/\*`,
      'Content-Type': `${this.type}`,
    };
  }


  next_range()
  {
    if (this.ranges.ranges.length == 1) {
      debug('Cannot start new range; only one requested.');
      this.stream.off('data', this.handlers['data']);
      return false;
    }

    var headers = this.next_range_headers();

    if (headers) {
      var header_buf = new stream_buf.WritableStreamBuffer();
      // We start a range with a boundary.
      header_buf.write(`\r\n--${this.range_boundary}\r\n`);

      // The we write the range headers.
      for (var header in headers) {
        header_buf.write(`${header}: ${headers[header]}\r\n`);
      }
      header_buf.write('\r\n');
      this.response.write(header_buf.getContents());
      debug('New range started.');
      return true;
    }

    // No headers means we're finishing the last range.
    this.response.write(`\r\n--${this.range_boundary}--\r\n`);
    debug('End of ranges sent.');
    this.stream.off('data', this.handlers['data']);
    return false;
  }


  on_open_ranges()
  {
    // File got opened, so we can set headers/status
    debug('Open succeeded:', this.name, this.type);

    this.response.header('Accept-Ranges', 'bytes');
    this.response.header('Content-Transfer-Encoding', 'binary');
    this.response.header('Content-Disposition', 'inline');

    // For single ranges, the content length should be the size of the
    // range. For multiple ranges, we don't send a content length
    // header.
    //
    // Similarly, the type is different whether or not there is more than
    // one range.
    if (this.ranges.ranges.length == 1) {
      this.response.writeHead(206, 'Partial Content', this.next_range_headers());
    }
    else {
      this.range_boundary = uuid.v4();
      var headers = {
        'Content-Type': `multipart/byteranges; boundary=${this.range_boundary}`,
      };
      this.response.writeHead(206, 'Partial Content', headers);
      this.next_range();
    }
  }

  on_data_ranges(chunk)
  {
    // Crap, node.js streams are stupid. No guarantee for seek support. Sure,
    // that makes node.js easier to implement, but offloads everything onto the
    // application developer.
    //
    // So, we skip chunks until our read position is within the range we want to
    // send at the moment. We're relying on ranges being in-order, which this
    // file's parser luckily (?) provides.
    //
    // The simplest optimization would be at ever range start to seek() to the
    // start.
    var chunk_range = [this.read_offset, this.read_offset + chunk.length - 1];
    debug('= Got chunk with byte range', chunk_range);
    while (true) {
      var req_range = this.ranges.ranges[this.range_index];
      if (!req_range) {
        break;
      }
      debug('Current requested range is', req_range);

      // No overlap in the chunk and requested range; don't write.
      if (chunk_range[1] < req_range[0] || chunk_range[0] > req_range[1]) {
        debug('Ignoring chunk; it is out of range.');
        break;
      }

      // Since there is overlap, find the segment that's entirely within the
      // chunk.
      var segment = [
        Math.max(chunk_range[0], req_range[0]),
        Math.min(chunk_range[1], req_range[1]),
      ];
      debug('Segment to send within chunk is', segment);

      // Normalize the segment to a chunk offset
      segment[0] -= this.read_offset;
      segment[1] -= this.read_offset;
      var len = segment[1] - segment[0] + 1;
      debug('Offsets into buffer are', segment, 'with length', len);

      // Write the slice that we want to write. We first create a buffer from the
      // chunk. Then we slice a new buffer from the same underlying ArrayBuffer,
      // starting at the original buffer's offset, further offset by the segment
      // start. The segment length bounds the end of our slice.
      var buf = Buffer.from(chunk, 'binary');
      this.response.write(Buffer.from(buf.buffer, buf.byteOffset + segment[0], len));

      // If the requested range is finished, we should start the next one.
      if (req_range[1] > chunk_range[1]) {
        debug('Chunk is finished, but the requested range is missing bytes.');
        break;
      }

      if (req_range[1] <= chunk_range[1]) {
        debug('Range is finished.');
        if (!this.next_range()) {
          break;
        }
      }
    }

    // Update read offset when chunk is finished.
    this.read_offset += chunk.length;
  }


  start()
  {
    // Register callbacks. Store them in a handlers object so we can
    // keep the bound version around for stopping to listen to events.
    this.handlers['error'] = this.on_error.bind(this);
    this.handlers['end'] = this.on_end.bind(this);

    if (!this.ranges) {
      this.handlers['open'] = this.on_open_no_range.bind(this);
      this.handlers['data'] = this.on_data_no_range.bind(this);
    }
    else {
      this.handlers['open'] = this.on_open_ranges.bind(this);
      this.handlers['data'] = this.on_data_ranges.bind(this);
    }

    for (var handler in this.handlers) {
      this.stream.on(handler, this.handlers[handler]);
    }
  }
};


function send(response, stream, opts, end_callback)
{
  var sender = new RangeSender(response, stream, opts, end_callback);
  sender.start();
};


/*****************************************************************************
 * Exports
 **/

module.exports =
{
  parse: parse,
  parseAsync: parseAsync,
  RangeSender: RangeSender,
  send: send,
};
