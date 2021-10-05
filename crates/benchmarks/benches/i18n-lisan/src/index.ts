import {PaymentMethodsAPI} from '@shopify/scripts-checkout-apis-ts';
import {t, lisan} from 'lisan';


type Payload = PaymentMethodsAPI.Payload;
type Output = PaymentMethodsAPI.Output;

export const main = (payload: Payload): Output => {
  return  {
    sortResponse: null,
    filterResponse: null,
    renameResponse:  {
      renameProposals: payload.input.paymentMethods.map((method, index) => {

        if (index % 2 == 0) {
          const dict = require(`./.lisan_out/dictionaries/es/main.js`);
          lisan.add(dict);
        } else {
          const dict = require(`./.lisan_out/dictionaries/fr/main.js`);
          lisan.add(dict);
        }

        return  {
          paymentMethod: method,
          name: t('payment_method.rename'), 
          renamed: true
        };
     })
    }
  };
};
