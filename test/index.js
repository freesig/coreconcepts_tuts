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

diorama.registerScenario("Test hello holo", async (s, t, { alice }) => {
  // Make a call to a Zome function
  // indicating the function, and passing it an input
  const result = await alice.call("hello", "hello_holo", {});
  // Make sure the result is ok.
  t.ok(result.Ok);

  // check for equality of the actual and expected results
  t.deepEqual(result, { Ok: 'Hello Holo' })
})

diorama.run()
