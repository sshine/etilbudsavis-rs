# API catalogue

The app is a Capacitor shell around a React/TanStack bundle. The bundle's
`entry-client-*.js` contains the whole client. It is minified but not
obfuscated; `nix run nixpkgs#prettier` makes it readable.

## Transport

| Purpose | Request |
| --- | --- |
| Read | `POST https://etilbudsavis.tjek.com/` |
| Command | `POST https://etilbudsavis.tjek.com/api/<operation>` |

Read body:

```json
{ "data": ["<base64 of [key, params]>"] }
```

The key is `base64(JSON.stringify([dataKey, params]))`, with object keys sorted
recursively. `params` is `null` for keys that take none. Responses are NDJSON;
each line is one requested record:

```json
{ "key": "...", "status": "success", "value": ... }
{ "key": "...", "status": "error", "error": { "status": 401, "message": "..." } }
```

Command bodies are plain JSON. A successful command may return `null`.

Headers:

| Header | Value |
| --- | --- |
| `x-api-key` | `590990e7ad62f206512ce390939ea3d3` (Android build) |
| `x-token` | account JWT, when signed in |
| `x-location` | `encodeURIComponent(JSON.stringify({country, geohash, source}))` |
| `content-type` | `application/json` |

The bundle also carries browser and iOS API keys, but the Android key is the
one this project uses. The keys are public in the bundle; the token is the
credential.

## Read data keys

### Catalogue

| Key | Params | Returns |
| --- | --- | --- |
| `groupedFronts` | `null` | `[[category, [front, ...]], ...]` |
| `fronts` | `{businessIds?, businessSlugs?, localBusinessIds?, coordinates?, hideInactive?, hideDisabled?, category?, keepStatic?, businessSort?, locationId?, pagination?}` | `[front, ...]` |
| `businesses` | `{businessIds?, category?, limit?}` | `[business, ...]` |
| `business` | `{slug}` | `business` |
| `businessDescription` | `{publicId}` | Markdown string |
| `publication` | `{publicationId}` | `{publication: metadata}` |
| `relatedPublications` | `{businessId, locationId?}` | `[publication, ...]` |
| `publicationViewer` | `{publicId, appVersion, apiKey, location, enableSync?, enableZoom?}` | A JavaScript viewer string; not modelled |

### Offers

| Key | Params | Returns |
| --- | --- | --- |
| `offers` | `{businessIds?, departmentSlug?, sources?, pagination?, sort?, searchTerm?, offerIds?, publicationIds?, storeIds?, categoryIds?, hideUpcoming?, minPrice?, maxPrice?, maxRadius?, coordinates?, locationId?}` | `{data: [offer], metadata}` |
| `offer` | `{publicId}` | `offer` |
| `offerProducts` | `{publicId}` | `[offerProduct, ...]` |
| `relatedOffers` | `{publicId, sources?}` | `[offer, ...]` |
| `departmentOffers` | same as `offers` | `{data: [{departmentSlug, offers}], metadata?}` |
| `forYou` | `null` | `{name, offers: [offer]}` |

### Stores and location

| Key | Params | Returns |
| --- | --- | --- |
| `stores` | `{publicationId?, offerId?, businessId?, businessIds?, searchTerm?, maxRadius?, pagination?, locationId?}` | `{data: [store], _metadata}` |
| `store` | `{storeId}` | `store` |
| `location` | `null` | `{latitude, longitude, geohash, city, country, mode}` |
| `locationInfo` | `{latitude, longitude}` | `{geohash, country}` |

### Search and discovery

| Key | Params | Returns |
| --- | --- | --- |
| `topSearches` | `{pagination}` | `{data: [{title, type, source}], _metadata}` |
| `searchBusiness` | `{searchTerm}` | `{business}` |
| `searchSEO` | `{searchTerm}` | `{data: {breadcrumb, parentCategories, adjacentCategories, childCategories, markdownDescription}}` |
| `search` (command) | `{query, types: ["search_query" or "shopping_list_item"]}` | `{_metadata: {size}, data: [searchResult]}` |
| `ads` | `{type, searchTerm?, creativeTypes?, id?}` | `{data: ad}`; out of scope |
| `surveys` | `null` | `{data: [survey]}`; out of scope |

### Videos

| Key | Params | Returns |
| --- | --- | --- |
| `videos` | `{limit?, search?}` | `{data: [video]}` |
| `video` | `{id}` | `{data: video}` |
| `videoIngredients` | `{videoPublicId}` | `{data: [ingredient]}` |
| `likedVideos` | `null` | `{data: [video]}` |
| `myVideos` | `null` | `[video]` |

### Account and lists

| Key | Params | Returns |
| --- | --- | --- |
| `auth` | `null` | `{person}` |
| `favorites` | `null` | `[business, ...]` |
| `offerAgents` | `null` | `[offerAgent, ...]` |
| `shoppingLists` | `null` | `[shoppingList, ...]` |
| `shoppingListItems` | `{shoppingListId}` | `[shoppingListItem, ...]` |
| `privacyPolicy` | `{locale}` | Markdown string |

## Command operations

| Operation | Body | Returns |
| --- | --- | --- |
| `auth-create-person` | `{apiKey}` | `{person, token}` |
| `auth-otp-initialize` | `{apiKey, email}` | `{otpId, ...}` |
| `auth-otp-finalize` | `{apiKey, otpId, otp}` | `{token, email, person?}` |
| `auth-terminate` | `null` | `null` |
| `auth-update-person` | `{apiKey, countryCode?, privacyPolicyConsented?, enabledNotifications?}` | person |
| `auth-delete-person-initialize` | `{apiKey}` | OTP start; out of scope |
| `auth-delete-person-finalize` | `{apiKey, otpId, otp}` | out of scope |
| `create-favorite` | `{businessPublicId}` | `null` |
| `delete-favorite` | `{businessPublicId}` | `null` |
| `offer-agent-create` | `{term, geohash, sorting?, maxRadius?, includeUpcoming, priceMin?, priceMax?, businessIds, isEmailNotificationsEnabled, isPushNotificationsEnabled}` | `{publicId}` |
| `offer-agent-update` | `{publicId, lastViewedAt}` | `null` |
| `offer-agent-delete` | `{publicId}` | `null` |
| `shopping-list-create-list` | `{name}` | `{id}` |
| `shopping-list-rename-list` | `{id, newName}` | `null` |
| `shopping-list-delete-list` | `{id}` | `null` |
| `shopping-list-leave-list` | `{id}` | `null` |
| `shopping-list-remove-collaborators` | `{id}` | `null` |
| `shopping-list-accept-invitation` | `{shareToken}` | `{id}` |
| `shopping-list-add-item` | `{shoppingListId, item}` | `null` |
| `shopping-list-bulk-add-items` | `{shoppingListId, items}` | `null` |
| `shopping-list-update-item` | `{shoppingListId, item}` | `null` |
| `shopping-list-bulk-update-items` | `{shoppingListId, items}` | `null` |
| `shopping-list-remove-item` | `{shoppingListId, itemId}` | `null` |
| `shopping-list-bulk-remove-items` | `{shoppingListId, itemIds}` | `null` |
| `shopping-list-auto-categorize-item` | `{shoppingListId, clientId}` | `{clientId, departmentSlug}` |
| `shopping-list-import-items-via-image` | `{file}` | `[{name, description, count}]`; out of scope |
| `video-like` | `{publicId}` | `null` |
| `video-unlike` | `{publicId}` | `null` |
| `my-video-create` | `{video, title, description}` | out of scope |
| `my-video-update` | `{id, title, description}` | out of scope |
| `my-video-delete` | `{id}` | out of scope |
| `upload-file` | `{mimetype, byteSize}` | upload form data; out of scope |
| `places-autocomplete` | `{query, sessionToken}` | places; unused |
| `places-details` | `{placeId, sessionToken}` | place; unused |
| `create-interaction` | `{id, type}` | tracking; unused |
| `answer-survey` | `{surveyId, answers}` | out of scope |
| `app-install-upsert` | `{locale_code, push_notification_service, push_notification_token, push_notification_supported_types}` | out of scope |

## Quirks

- `offers` uses `metadata`; `stores`, `topSearches`, and `search` use
  `_metadata`.
- Timestamps appear as RFC 3339, as `+0000` offsets without a colon, and as
  JavaScript `Date#toString` in publication metadata. The client accepts all
  three where a value is useful.
- Prices are numbers, sometimes with long floating-point tails such as
  `99.74999999999999`; the client keeps `f64`.
- Adding a shopping-list item requires `createdAt` and a client id. A comment
  is sent as `comment` on add and read back as `shortDescription`; updates use
  `shortDescription`.
- Null optional fields are often rejected with 422. The Rust request types use
  `skip_serializing_if` rather than sending `null`.
- `groupedFronts` is large (about 160 KiB); the MCP catalogue tool pages it.
