# Configuration parameters

## Env

## Topology files

### certissuer

Create the file `certissuer_pki` in the Coup D'Oeil configuration area. Give domain access to `CorePki`. 

Parameters:

* urls: Comma separated list of certissuer urls to be used by the domain Pki, this is added to the environment parameter list. Example: http://url1.com:2080,http://url2.com:2080
* user_period: int, number of seconds of validity (e.g. 86_400 is one day, default is 6 months)
