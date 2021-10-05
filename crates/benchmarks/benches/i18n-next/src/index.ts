import {PaymentMethodsAPI} from '@shopify/scripts-checkout-apis-ts';
import i18next from 'i18next';
import * as i18n_es from './locales/es.json';
import * as i18n_fr from './locales/fr.json';

i18next.init({
  initImmediate: false,
  fallbackLng: 'en',
  resources: {
      en: { translation: i18n_es },
      fr: { translation: i18n_fr },
  }
});


type Payload = PaymentMethodsAPI.Payload;
type Output = PaymentMethodsAPI.Output;

export const main = (payload: Payload): Output => {
  
  return {
    sortResponse: null,
    filterResponse: null,
    renameResponse: {
      renameProposals: payload.input.paymentMethods.map((method, index) => {

        if (index % 2 === 0) {
          i18next.changeLanguage('es')
        } else {
          i18next.changeLanguage('fr')
        }

        return  {
          paymentMethod: method,
          name: i18next.t('payment_method.rename'),
          renamed: true
        };
      }),
    }
  };
};
