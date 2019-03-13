'use strict';

const mocha = require('mocha');
const expect = require('chai').expect;

const mutual = require('joystream/protocols/mutual');
const keys = require('joystream/crypto/keys');

function run_protocol(auth1, auth2, done)
{
  // Auth1/key1 initiates.
  auth1.initiate((err, type, challenge) => {
    expect(err).to.be.null;
    expect(type).to.equal(mutual.MSG_CHALLENGE);
    expect(auth1.peer_authenticated).to.be.false;

    // Consume message on auth2
    auth2.consume(type, challenge, (err, type, response) => {
      expect(err).to.be.null;
      expect(type).to.equal(mutual.MSG_RESPONSE);
      expect(auth2.peer_authenticated).to.be.false;

      // Consume response on auth1
      auth1.consume(type, response, (err, type, finalize) => {
        expect(err).to.be.null;
        expect(type).to.equal(mutual.MSG_FINALIZE);
        expect(auth1.peer_authenticated).to.be.true;

        // Consume finalize on auth2
        auth2.consume(type, finalize, (err) => {
          expect(err).to.be.null;
          expect(auth2.peer_authenticated).to.be.true;

          // Both authenticated, awesome.
          done();
        });
      });
    });
  });
}

describe('protocols/mutual', function()
{
  it('mutually authenticates two peers', function(done)
  {
    const key1 = keys.key_pair();
    const key2 = keys.key_pair();

    var auth1 = new mutual.MutualAuthenticator(key1, key2.pubKey, 8);
    var auth2 = new mutual.MutualAuthenticator(key2, key1.pubKey, 8);

    run_protocol(auth1, auth2, done);
  });

  it('describes a message range', function()
  {
    expect(mutual.MutualAuthenticator).to.have.property('MESSAGE_RANGE');
    expect(mutual.MutualAuthenticator.MESSAGE_RANGE).to.be.have.lengthOf(2);
  });

  describe('failures with a bad key pair', function()
  {
    it ('it fails if the first peer has a bad key pair', function(done)
    {
      const key1 = keys.key_pair();
      // Change private key of key1
      key1.privKey[0] += 1;

      const key2 = keys.key_pair();

      var auth1 = new mutual.MutualAuthenticator(key1, key2.pubKey, 8);
      var auth2 = new mutual.MutualAuthenticator(key2, key1.pubKey, 8);

      // Auth1/key1 initiates.
      auth1.initiate((err, type, challenge) => {
        expect(err).to.be.null;
        expect(type).to.equal(mutual.MSG_CHALLENGE);
        expect(auth1.peer_authenticated).to.be.false;

        // Consume message on auth2
        auth2.consume(type, challenge, (err, type, response) => {
          expect(err).to.be.null;
          expect(type).to.equal(mutual.MSG_RESPONSE);
          expect(auth2.peer_authenticated).to.be.false;

          // Consume response on auth1
          auth1.consume(type, response, (err, type, finalize) => {
            // Since the private key of peer1 does not match their public
            // key, they experience the failure in authentication - the
            // response challenge can't be descrypted.
            expect(err).not.to.be.null;
            done();
          });
        });
      });
    });

    it ('it fails if the second peer has a bad key pair', function(done)
    {
      const key1 = keys.key_pair();
      const key2 = keys.key_pair();
      // Change private key of key2
      key2.privKey[0] += 1;

      var auth1 = new mutual.MutualAuthenticator(key1, key2.pubKey, 8);
      var auth2 = new mutual.MutualAuthenticator(key2, key1.pubKey, 8);

      // Auth1/key1 initiates.
      auth1.initiate((err, type, challenge) => {
        expect(err).to.be.null;
        expect(auth1.peer_authenticated).to.be.false;

        // Consume message on auth2
        auth2.consume(type, challenge, (err, type, response) => {
          expect(err).to.be.null;
          expect(auth2.peer_authenticated).to.be.false;

          // Consume response on auth1
          auth1.consume(type, response, (err, type, finalize) => {
            // Again, it's the initiating peer that experiences the
            // authentication failure. In this case, though, it's because
            // the other peer sent a bad response.
            expect(err).not.to.be.null;
            done();
          });
        });
      });
    });
  });
});
