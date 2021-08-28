const mongoose = require('mongoose');

const { toJSON } = require('./plugins');

const schemaOptions = {
  timestamps: {
    createdAt: 'createdDate',
    updatedAt: 'lastModifiedDate',
  },
};

const tleSchema = mongoose.Schema({
  lineOne: {
    type: String,
    trim: true,
    required: true,
  },
  lineTwo: {
    type: String,
    trim: true,
    required: true,
  },
});

const satelliteSchema = mongoose.Schema(
  {
    name: {
      type: String,
      trim: true,
    },
    tle: {
      type: tleSchema,
      required: true,
    },
    satrec: {
      type: Object,
    },
  },
  schemaOptions
);

satelliteSchema.plugin(toJSON);

/**
 * @typedef Satellite
 */
const Satellite = mongoose.model('Satellite', satelliteSchema);

module.exports = Satellite;
