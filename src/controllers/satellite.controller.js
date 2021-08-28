const httpStatus = require('http-status');
const mongoose = require('mongoose');

const ApiError = require('../utils/ApiError');
const catchAsync = require('../utils/catchAsync');
const { satelliteService } = require('../services');
const { logger } = require('../config');

const createSatellitesFromTLE = catchAsync(async (req, res) => {
  const satellites = await satelliteService.createSatellitesFromTLE(req.files.tle);
  res.status(satellites);
});

const createSatellite = catchAsync(async (req, res) => {
  const satellite = await satelliteService.createSatellite(req.body);
  res.status(httpStatus.CREATED).send(satellite);
});

const listSatellites = catchAsync(async (req, res) => {
  // TODO Add filtering and pagination options in a production-ready version of this server so the API can scale
  const satellitesList = await satelliteService.listSatellites();
  res.send(satellitesList);
});

const getSatellite = catchAsync(async (req, res) => {
  logger.info(req.params);
  const satellite = await satelliteService.getSatelliteById(mongoose.Types.ObjectId(req.params.id));
  if (!satellite) {
    throw new ApiError(httpStatus.NOT_FOUND, 'Satellite not found');
  }
  res.send(satellite);
});

const updateSatellite = catchAsync(async (req, res) => {
  const updatedSatellite = await satelliteService.updateSatelliteById(mongoose.Types.ObjectId(req.params.id), req.body);
  res.send(updatedSatellite);
});

const deleteSatellite = catchAsync(async (req, res) => {
  await satelliteService.deleteSatelliteById(mongoose.Types.ObjectId(req.params.id));
  res.status(httpStatus.NO_CONTENT).send();
});

const getNextVisible = catchAsync(async (req, res) => {
  res.send(httpStatus.NOT_IMPLEMENTED);
});

const getOverhead = catchAsync(async (req, res) => {
  res.send(httpStatus.NOT_IMPLEMENTED);
});

module.exports = {
  createSatellite,
  createSatellitesFromTLE,
  listSatellites,
  getSatellite,
  updateSatellite,
  deleteSatellite,
  getNextVisible,
  getOverhead,
};
