const faker = require('faker');

const { Sattelite } = require('../../../src/models');

describe('Sattelite model', () => {
  describe('Sattelite validation', () => {
    let newSattelite;
    beforeEach(() => {
      newSattelite = {
        buildingCode: faker.address.stateAbbr(),
        description: faker.lorem.sentence(),
      };
    });

    test('should correctly validate a valid maintenance request', async () => {
      await expect(new Sattelite(newSattelite).validate()).resolves.toBeUndefined();
    });
    // TODO add more tests with invalid data to ensure proper handling
  });
});
