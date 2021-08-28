const faker = require('faker');

const { Satellite } = require('../../../src/models');

describe('Satellite model', () => {
  describe('Satellite validation', () => {
    let newSatellite;
    beforeEach(() => {
      newSatellite = {
        buildingCode: faker.address.stateAbbr(),
        description: faker.lorem.sentence(),
      };
    });

    test('should correctly validate a valid maintenance request', async () => {
      await expect(new Satellite(newSatellite).validate()).resolves.toBeUndefined();
    });
    // TODO add more tests with invalid data to ensure proper handling
  });
});
