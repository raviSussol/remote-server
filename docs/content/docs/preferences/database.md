+++
title = "Database"
description = "Preference related database details"
date = 2021-10-28T18:00:00+00:00
updated = 2021-10-28T18:00:00+00:00
draft = false
weight = 51
sort_by = "weight"
template = "docs/page.html"

[extra]
toc = true
+++

# Database & Schema

Preferences are spread over 5 tables. A Preference table defines the attribute for a preference while the variants: user/store/site, define the values for each of the entities.

### Preference

| Field name              | Data type | Constraint  | Description                                                                                                           |
| ----------------------- | --------- | ----------- | --------------------------------------------------------------------------------------------------------------------- |
| id                      | uuid      | Primary key |                                                                                                                       |
| is_centrally_controlled | boolean   |             | When this is true the value for the preference comes from the default value rather than the preference table variant. |
| key                     | string    |             | Simple key to identify this preference.                                                                               |
| description             | string    |             | Simple description of what this preference does.                                                                      |
| data_type               | string    |             | Either number / text / boolean detailing which type of data this preference should be.                                |
| default                 | object    |             | A default value for this preference. Note it is always an object { value: number \| string \| boolean }               |

### Server Preference

| Field name    | Data type | Constraint  | Description                                                                                     |
| ------------- | --------- | ----------- | ----------------------------------------------------------------------------------------------- |
| id            | uuid      | Primary key |                                                                                                 |
| preference_id | string    | Foreign key | The related [Preference] record                                                                 |
| default       | object    |             | A value for this preference. Note it is always an object { value: number \| string \| boolean } |

### Site Preference

| Field name    | Data type | Constraint  | Description                                                                                     |
| ------------- | --------- | ----------- | ----------------------------------------------------------------------------------------------- |
| id            | uuid      | Primary key |                                                                                                 |
| preference_id | string    | Foreign key | The related [Preference] record                                                                 |
| default       | object    |             | A value for this preference. Note it is always an object { value: number \| string \| boolean } |
| site_id       | string    | Foreign key | The related [Site] record                                                                       |

### User Preference

| Field name    | Data type | Constraint  | Description                                                                                     |
| ------------- | --------- | ----------- | ----------------------------------------------------------------------------------------------- |
| id            | uuid      | Primary key |                                                                                                 |
| preference_id | string    | Foreign key | The related [Preference] record                                                                 |
| default       | object    |             | A value for this preference. Note it is always an object { value: number \| string \| boolean } |
| user_id       | string    | Foreign key | The related [User] record                                                                       |

### Store Preference

| Field name    | Data type | Constraint  | Description                                                                                     |
| ------------- | --------- | ----------- | ----------------------------------------------------------------------------------------------- |
| id            | uuid      | Primary key |                                                                                                 |
| preference_id | string    | Foreign key | The related [Preference] record                                                                 |
| default       | object    |             | A value for this preference. Note it is always an object { value: number \| string \| boolean } |
| store_id      | string    | Foreign key | The related [Store] record                                                                      |
