import {
  AddressableEntityWrapper,
  CasperServiceByJsonRPC,
  NamedKey,
  type CLPublicKey,
  type GetDeployResult,
  type StoredValue
} from 'casper-js-sdk';

type Account = {
  addressableEntity: AddressableEntityWrapper;
  namedKeys: NamedKey[];
};

export const getAccountInfo = async (
  nodeAddress: string,
  publicKey: CLPublicKey
): Promise<Account> => {
  const client = new CasperServiceByJsonRPC(nodeAddress);
  const { AddressableEntity } = await client.getEntity({
    AccountHash: publicKey.toAccountHash().toFormattedString()
  });
  if (!AddressableEntity) throw Error('Not found account');
  let namedKeys = AddressableEntity.named_keys;
  AddressableEntity.entity;
  return {
    addressableEntity: AddressableEntity,
    namedKeys
  };
};

export const findKeyFromAccountNamedKeys = (
  account: Account,
  name: string
): string => {
  const key = account.namedKeys.find(namedKey => namedKey.name === name)?.key;

  if (!key) throw Error(`Not found key: ${name}`);

  return key;
};

export const sleep = async (ms: number): Promise<void> => {
  // eslint-disable-next-line no-promise-executor-return
  await new Promise(resolve => setTimeout(resolve, ms));
};

export const expectDeployResultToSuccess = (result: GetDeployResult): void => {
  if (
    result.execution_info &&
    result.execution_info.execution_result &&
    'Version2' in result.execution_info.execution_result
  ) {
    let v2 = result.execution_info.execution_result.Version2;

    expect(v2.error_message).toBeNull();
  } else {
    fail('Not found Version2 in execution_result');
  }
};
