
import { Event } from './types';

const { driverNew, insertEvent } = require('../bin/arque-driver.node');
export class ArqueDriver {

  private driver: any;

  constructor(endpoint: string) {
    this.driver = driverNew(endpoint);

  }

  async insertEvent(event: Event) {
     return insertEvent(this.driver, event);
  }

}