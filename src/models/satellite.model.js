const mongoose = require('mongoose');

const { toJSON } = require('./plugins');

const schemaOptions = {
  timestamps: {
    createdAt: 'createdDate',
    updatedAt: 'lastModifiedDate',
  },
};

const satelliteSchema = mongoose.Schema(
  {
    name: {
      type: String,
      trim: true,
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
