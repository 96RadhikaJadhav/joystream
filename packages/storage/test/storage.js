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
const expect = require('chai').expect;
const fs = require('fs');
const temp = require('temp').track();

const storage = require('@joystream/storage/storage');
const repository = require('@joystream/storage/repository');

function matchDirent(name, matcher)
{
  return (entry) => {
    return (entry.name == name && matcher(entry));
  };
}


describe('storage/storage', function()
{
  // Unique prefix per test
  var prefix;
  beforeEach(() => {
    prefix = temp.mkdirSync('joystream-storage-test');
  });

  function new_storage(pool_size = undefined)
  {
    var s = new storage.Storage(prefix, pool_size);
    expect(s).to.be.an.instanceof(storage.Storage);
    if (pool_size) {
      expect(s.pool_size).to.equal(pool_size);
    }
    return s;
  }

  describe('creation', function()
  {
    it('can create a new storage structure', function()
    {
      var s = new_storage();
      expect(typeof s.id).to.equal('string');
      expect(s.id).to.have.lengthOf(36);
    });

    it('can re-use an existing storage instance', function()
    {
      var s1 = new_storage();
      var s2 = new_storage();
      expect(s1.id).to.equal(s2.id);
      expect(s1.base_path).to.equal(s2.base_path);
    });

    it('creates the storage directory layout', function()
    {
      var s = new_storage();
      const dirent = fs.readdirSync(s.base_path, { withFileTypes: true });
      expect(dirent.some(matchDirent('keys', e => e.isDirectory()))).to.be.true;
      expect(dirent.some(matchDirent('repos', e => e.isDirectory()))).to.be.true;
      expect(dirent.some(matchDirent('id', e => e.isFile()))).to.be.true;

    });
  });

  describe('managemnet', function()
  {
    it('cannot return non-existent repositories', function()
    {
      var s = new_storage();
      var r = s.get('foo');
      expect(r).to.be.undefined;
    });

    it('creates repositories', function()
    {
      var s = new_storage();
      var res = s.create();
      expect(res.repo).to.be.an.instanceof(repository.Repository);
      expect(res.id).to.have.lengthOf(36);
    });

    it('returns created repositories', function(done)
    {
      var s = new_storage();
      var res = s.create();
      expect(res.repo).to.be.an.instanceof(repository.Repository);
      expect(res.id).to.have.lengthOf(36);

      res.repo.on('ready', () => {
        var repo = s.get(res.id);
        expect(repo).to.equal(res.repo); // Strict equal, yay!

        done();
      });
    });

    it('re-initialized created repositories', function(done)
    {
      var s = new_storage();
      var res = s.create();
      expect(res.repo).to.be.an.instanceof(repository.Repository);
      expect(res.id).to.have.lengthOf(36);

      res.repo.on('ready', () => {
        // Force flushing of the LRU pool
        s.pool.clear();

        var repo = s.get(res.id);
        expect(repo).to.be.an.instanceof(repository.Repository);

        done();
      });
    });
  });

  describe('templating', function()
  {
    it('can create a repository from a function template', function(done)
    {
      var s = new_storage();
      var res = s.create(undefined, (repo, commit) => {
        repo.open('/foo', 'w', (err, mime, stream) => {
          stream.write('Hello, world!');
          stream.end(undefined, undefined, commit);
        });
      }, (err, id, repo) => {
        // At this point, we can check the repo for a file list.
        repo.list('/', (err, files) => {
          expect(files).to.have.lengthOf(1);
          expect(files[0]).to.equal('foo');
          done();
        });
      });
    });

    it('can create a repository from a path template', function(done)
    {
      var s = new_storage();
      var res = s.create(undefined, './test/template', (err, id, repo) => {
        // At this point, we can check the repo for a file list.
        repo.list('/', (err, files) => {
          expect(files).to.have.lengthOf(2);
          expect(files).to.include('foo');
          expect(files).to.include('bar');
          // ignore symlink: expect(files).to.include('quux');

          repo.list('/foo', (err, files) => {
            expect(files).to.have.lengthOf(1);
            expect(files).to.include('baz');
            done();
          });
        });
      });
    });
  });
});
