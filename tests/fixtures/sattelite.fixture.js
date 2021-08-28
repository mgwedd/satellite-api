const mongoose = require('mongoose');
const faker = require('faker');

const Satellite = require('../../src/models/satellite.model');

const satelliteOne = {
  _id: mongoose.Types.ObjectId(),
  buildingCode: faker.address.stateAbbr(),
  description: faker.lorem.sentence(),
};

const satelliteTwo = {
  _id: mongoose.Types.ObjectId(),
  buildingCode: faker.address.stateAbbr(),
  description: faker.lorem.sentence(),
};

const insertSatellites = async (satellites) => {
  await Satellite.insertMany(satellites);
};

module.exports = {
  satelliteOne,
  satelliteTwo,
  insertSatellites,
};
