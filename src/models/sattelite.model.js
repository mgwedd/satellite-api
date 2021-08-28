const mongoose = require('mongoose');

const { toJSON } = require('./plugins');

const schemaOptions = {
  timestamps: {
    createdAt: 'createdDate',
    updatedAt: 'lastModifiedDate',
  },
};

const satteliteSchema = mongoose.Schema(
  {
    name: {
      type: String,
      trim: true,
    },
    description: {
      type: String,
      trim: true,
      required: true,
    },
    currentStatus: {
      type: String,
      trim: true,
      enum: ['NotApplicable', 'Created', 'InProgress', 'Complete', 'Canceled'],
      default: 'Created',
    },
  },
  schemaOptions
);

satteliteSchema.plugin(toJSON);

/**
 * @typedef Sattelite
 */
const Sattelite = mongoose.model('Sattelite', satteliteSchema);

module.exports = Sattelite;
