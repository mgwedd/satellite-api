const httpStatus = require('http-status');

const { Sattelite } = require('../models');
const ApiError = require('../utils/ApiError');

/**
 * Bulk create sattelites from TLE data
 * @param {Object} tles
 * @returns {Promise<Array.Sattelite>}
 */
const bulkCreateSattelites = async (tles) => {
  return Sattelite.create(tles);
};

/**
 * List sattelites
 * @returns {Promise<QueryResult>}
 */
const listSattelites = async () => {
  const sattelites = await Sattelite.find();
  return sattelites;
};

/**
 * Get sattelite by id
 * @param {ObjectId} id
 * @returns {Promise<Sattelite>}
 */
const getSatteliteById = async (id) => {
  return Sattelite.findById(id);
};

/**
 * Update sattelite by id
 * @param {ObjectId} id
 * @param {Object} updateBody
 * @returns {Promise<Sattelite>}
 */
const updateSatteliteById = async (id, updateBody) => {
  const sattelite = await getSatteliteById(id);
  if (!sattelite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Sattelite not found');
  }
  Object.assign(sattelite, updateBody);
  await sattelite.save();
  return sattelite;
};

/**
 * Delete sattelite by id
 * @param {ObjectId} id
 * @returns {Promise<Sattelitet>}
 */
const deleteSatteliteById = async (id) => {
  const sattelite = await getSatteliteById(id);
  if (!sattelite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Sattelite not found');
  }
  await sattelite.remove();
  return sattelite;
};

module.exports = {
  bulkCreateSattelites,
  listSattelites,
  getSatteliteById,
  updateSatteliteById,
  deleteSatteliteById,
};
