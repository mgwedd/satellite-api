const mongoose = require('mongoose');
const faker = require('faker');

const Sattelite = require('../../src/models/sattelite.model');

const satteliteOne = {
  _id: mongoose.Types.ObjectId(),
  buildingCode: faker.address.stateAbbr(),
  description: faker.lorem.sentence(),
};

const satteliteTwo = {
  _id: mongoose.Types.ObjectId(),
  buildingCode: faker.address.stateAbbr(),
  description: faker.lorem.sentence(),
};

const insertSattelites = async (sattelites) => {
  await Sattelite.insertMany(sattelites);
};

module.exports = {
  satteliteOne,
  satteliteTwo,
  insertSattelites,
};
