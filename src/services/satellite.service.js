const httpStatus = require('http-status');

const { Satellite } = require('../models');
const ApiError = require('../utils/ApiError');

/**
 * Bulk create satellites from TLE data
 * @param {Object} tles
 * @returns {Promise<Array.Satellite>}
 */
const bulkCreateSatellites = async (tles) => {
  return Satellite.create(tles);
};

/**
 * List satellites
 * @returns {Promise<QueryResult>}
 */
const listSatellites = async () => {
  const satellites = await Satellite.find();
  return satellites;
};

/**
 * Get satellite by id
 * @param {ObjectId} id
 * @returns {Promise<Satellite>}
 */
const getSatelliteById = async (id) => {
  return Satellite.findById(id);
};

/**
 * Update satellite by id
 * @param {ObjectId} id
 * @param {Object} updateBody
 * @returns {Promise<Satellite>}
 */
const updateSatelliteById = async (id, updateBody) => {
  const satellite = await getSatelliteById(id);
  if (!satellite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Satellite not found');
  }
  Object.assign(satellite, updateBody);
  await satellite.save();
  return satellite;
};

/**
 * Delete satellite by id
 * @param {ObjectId} id
 * @returns {Promise<Satellitet>}
 */
const deleteSatelliteById = async (id) => {
  const satellite = await getSatelliteById(id);
  if (!satellite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Satellite not found');
  }
  await satellite.remove();
  return satellite;
};

module.exports = {
  bulkCreateSatellites,
  listSatellites,
  getSatelliteById,
  updateSatelliteById,
  deleteSatelliteById,
};
