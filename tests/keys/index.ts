import { Keypair } from '@solana/web3.js';
import * as fs from 'fs';

const DEAL_CONTRACT_KEYS_PATH = process.env.DEAL_CONTRACT_KEYS_PATH;

export const payerKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/upgrade_authority.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const mintAuthorityKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/mint_authority.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()

export const clientKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/client.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const executorKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/executor.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const checkerKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/checker.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const serviceKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/service.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const serviceFeeMintKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/service_fee_mint.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()
export const holderMintKp: Keypair = (() => {
  let secret: Uint8Array = JSON.parse(fs.readFileSync(`${DEAL_CONTRACT_KEYS_PATH}/holder_mint.json`, 'utf-8'))
  return Keypair.fromSecretKey(new Uint8Array(secret))
})()

