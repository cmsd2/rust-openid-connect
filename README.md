# OpenId Connect Server and Libraries

This is a project to create libraries for OpenId Connect, implementing 
both the identity provider and relying party functionality.

See the [OpenId Connect](http://openid.net/connect) website for details and specifications.

## Parts

Signing and encryption using JWTs is handled by my fork of [jsonwebtoken](https://github.com/cmsd2/rust-jwt)

Oauth2 and OpenID Connect endpoints are provided by this library.

MVC style login, user registration etc pages are also implemented here.