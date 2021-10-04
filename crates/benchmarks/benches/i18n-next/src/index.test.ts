import {main} from './';
import * as input from './input.json';

describe('i18 benches', () => {
  it('renames all the payment methods', () => {
    const result = main(input as any);
    const proposals = result.renameResponse?.renameProposals.map(proposal => proposal.name);
    expect(proposals).toHaveLength(2);
    expect(proposals).toEqual(['Forma de pago', 'Mode de paiment']);
  });
});
