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
const sinon = require('sinon');

const { RuntimeApi } = require('@joystream/runtime-api');

describe('Identities', () => {
  var api;
  before(async () => {
    api = await RuntimeApi.create();
  });

  it('creates role keys', async () => {
    const key = await api.identities.createRoleKey('foo', 'bar');
    expect(key).to.have.property('type', 'ed25519');
    expect(key.getMeta().name).to.include('foo');
    expect(key.getMeta().name).to.include('bar');
  });

  it('Can import keys', async () => {
    // Unlocked keys can be imported without asking for a passphrase
    await api.identities.loadUnlock('test/data/edwards_unlocked.json');

    // Edwards and schnorr keys should unlock
    const passphrase_stub = sinon.stub(api.identities, 'askForPassphrase').callsFake(_ => 'asdf');
    await api.identities.loadUnlock('test/data/edwards.json');
    await api.identities.loadUnlock('test/data/schnorr.json');
    passphrase_stub.restore();

    // Except if the wrong passphrase is given
    const passphrase_stub_bad = sinon.stub(api.identities, 'askForPassphrase').callsFake(_ => 'bad');
    expect(async () => {
      await api.identities.loadUnlock('test/data/edwards.json');
    }).to.throw;
    passphrase_stub_bad.restore();
  });
});
