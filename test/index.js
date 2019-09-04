const path = require('path')
const tape = require('tape')

const { Diorama, tapeExecutor, backwardCompatibilityMiddleware } = require('@holochain/diorama')

process.on('unhandledRejection', error => {
  // Will print "unhandledRejection err is not defined"
  console.error('got unhandledRejection:', error);
});

const dnaPath = path.join(__dirname, "../dist/hello_holo.dna.json")
const dna = Diorama.dna(dnaPath, 'hello_holo')

const diorama = new Diorama({
  instances: {
    alice: dna,
    bob: dna,
  },
  bridges: [],
  debugLog: false,
  executor: tapeExecutor(require('tape')),
  middleware: backwardCompatibilityMiddleware,
})

// Register a test scenario that checks `hello_holo()`
// returns the correct value.
diorama.registerScenario("Test hello holo", async (s, t, { alice, bob }) => {
  // Make a call to the `hello_holo` Zome function
  // passing no arguments.
  const result = await alice.call("hello", "hello_holo", {});
  // Make sure the result is ok.
  t.ok(result.Ok);

  // Check that the result matches what you expected.
  t.deepEqual(result, { Ok: 'Hello Holo' })
  
  const create_result = await alice.call("hello", "create_person", {"person": { "name" : "Alice" }});
  // Make sure the result is ok.
  t.ok(create_result.Ok);
  
  const retrieve_result = await alice.call("hello", "retrieve_person", {"address": create_result.Ok});
  // Make sure the result is ok.
  t.ok(retrieve_result.Ok);
  
  t.deepEqual(retrieve_result, { Ok: { App: [ 'person', '{"name":"Alice"}' ] }})
  
  const bob_retrieve_result = await bob.call("hello", "retrieve_person", {"address": create_result.Ok});
  // Make sure the result is ok.
  t.ok(bob_retrieve_result.Ok);
  
  t.deepEqual(bob_retrieve_result, { Ok: { App: [ 'person', '{"name":"Alice"}' ] }})
})

diorama.run()
