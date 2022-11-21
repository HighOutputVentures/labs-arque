import { connect, generateObjectId } from 'arque-node-driver';

async function main() {
  const driver = connect({ url: 'tcp://localhost:4000' });

  const id = generateObjectId();

  const event: Event = // generate Event;

  await driver.insertEvent({ event });
}

main();