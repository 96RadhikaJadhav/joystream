/*
 * This file is part of the storage node for the Joystream project.
 * Copyright (C) 2019 Joystream Contributors
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

'use strict';

const mocha = require('mocha');
const chai = require('chai');
const chai_as_promised = require('chai-as-promised');
chai.use(chai_as_promised);
const expect = chai.expect;

const temp = require('temp').track();

const fs = require('fs');

const { Storage } = require('@joystream/storage');

function write(store, content_id, contents, callback)
{
  store.open('content_id', 'w')
    .then((stream) => {

      stream.on('committed', callback);
      stream.write(contents);
      stream.end();
    })
    .catch((err) => {
      expect.fail(err);
    });
}

function read_all(stream)
{
  var data = Buffer.alloc(0);
  var buffer;
  do {
    buffer = stream.read();
    if (buffer) {
      data = Buffer.concat([data, buffer]);
    }
  } while (buffer);
  return data;
}


function create_known_object(content_id, contents, callback)
{
  var hash;
  Storage.create({
    resolve_content_id: () => {
      return hash;
    },
  })
  .then((store) => {
    write(store, content_id, contents, (the_hash) => {
      hash = the_hash;

      callback(store, hash);
    });
  })
  .catch((err) => {
    expect.fail(err);
  });
}

describe('storage/storage', () => {
  var storage;
  before(async () => {
    storage = await Storage.create({ timeout: 1500 });
  });

  describe('open()', () => {
    it('can write a stream', (done) => {
      write(storage, 'foobar', 'test-content', (hash) => {
        expect(hash).to.not.be.undefined;
        // We know that 'test-content' creates this hash
        expect(hash).to.equal('QmfCXRe6PP21EWFfL5byA8bvX4KPPrzuGykh9GXBsEe9Kk');
        done();
      });
    });

    it('can read a stream', (done) => {
      const contents = 'test-for-reading';
      create_known_object('foobar', contents, (store, hash) => {
        store.open('foobar', 'r')
          .then((stream) => {
            const data = read_all(stream);
            expect(Buffer.compare(data, Buffer.from(contents))).to.equal(0);
            done();
          })
          .catch((err) => {
            expect.fail(err);
          });
      });
    });

    it('detects the file type of a read stream', (done) => {
      const contents = fs.readFileSync('../../banner.svg');
      create_known_object('foobar', contents, (store, hash) => {
        store.open('foobar', 'r')
          .then((stream) => {
            const data = read_all(stream);
            expect(Buffer.compare(data, contents)).to.equal(0);
            expect(stream).to.have.property('file_info');

            // application/xml+svg would be better, but this is good-ish.
            expect(stream.file_info).to.have.property('mime_type', 'application/xml');
            expect(stream.file_info).to.have.property('ext', 'xml');
            done();
          })
          .catch((err) => {
            expect.fail(err);
          });
      });

    });
  });

  describe('stat()', () => {
    it('times out for unknown content', async () => {
      const content = Buffer.from('this-should-not-exist');
      const x = await storage.ipfs.add(content, { onlyHash: true });
      const hash = x[0].hash;

      // Try to stat this entry, it should timeout.
      expect(storage.stat(hash)).to.eventually.be.rejectedWith('timed out');
    });

    it('returns stats for a known object', (done) => {
      create_known_object('foobar', 'stat-test', (store, hash) => {
        expect(store.stat(hash)).to.eventually.have.property('DataSize', 15);
        done();
      });
    });
  });

  describe('size()', () => {
    it('times out for unknown content', async () => {
      const content = Buffer.from('this-should-not-exist');
      const x = await storage.ipfs.add(content, { onlyHash: true });
      const hash = x[0].hash;

      // Try to stat this entry, it should timeout.
      expect(storage.size(hash)).to.eventually.be.rejectedWith('timed out');
    });

    it('returns the size of a known object', (done) => {
      create_known_object('foobar', 'stat-test', (store, hash) => {
        expect(store.size(hash)).to.eventually.equal(15);
        done();
      });
    });
  });
});
