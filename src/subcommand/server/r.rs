use super::*;

pub(super) async fn blockhash(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<Json<String>> {
  task::block_in_place(|| {
    Ok(Json(
      index
        .block_hash(None)?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    ))
  })
}

pub(super) async fn blockhash_at_height(
  Extension(index): Extension<Arc<Index>>,
  Path(height): Path<u32>,
) -> ServerResult<Json<String>> {
  task::block_in_place(|| {
    Ok(Json(
      index
        .block_hash(Some(height))?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    ))
  })
}

pub(super) async fn block_hash_from_height_string(
  Extension(index): Extension<Arc<Index>>,
  Path(height): Path<u32>,
) -> ServerResult<String> {
  task::block_in_place(|| {
    Ok(
      index
        .block_hash(Some(height))?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    )
  })
}

pub(super) async fn blockhash_string(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<String> {
  task::block_in_place(|| {
    Ok(
      index
        .block_hash(None)?
        .ok_or_not_found(|| "blockhash")?
        .to_string(),
    )
  })
}

pub(super) async fn blockheight_string(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<String> {
  task::block_in_place(|| {
    Ok(
      index
        .block_height()?
        .ok_or_not_found(|| "blockheight")?
        .to_string(),
    )
  })
}

pub(super) async fn blockinfo(
  Extension(index): Extension<Arc<Index>>,
  Path(DeserializeFromStr(query)): Path<DeserializeFromStr<query::Block>>,
) -> ServerResult<Json<api::BlockInfo>> {
  task::block_in_place(|| {
    let hash = match query {
      query::Block::Hash(hash) => hash,
      query::Block::Height(height) => index
        .block_hash(Some(height))?
        .ok_or_not_found(|| format!("block {height}"))?,
    };

    let header = index
      .block_header(hash)?
      .ok_or_not_found(|| format!("block {hash}"))?;

    let info = index
      .block_header_info(hash)?
      .ok_or_not_found(|| format!("block {hash}"))?;

    let stats = index
      .block_stats(info.height.try_into().unwrap())?
      .ok_or_not_found(|| format!("block {hash}"))?;

    Ok(Json(api::BlockInfo {
      average_fee: stats.avg_fee.to_sat(),
      average_fee_rate: stats.avg_fee_rate.to_sat(),
      bits: header.bits.to_consensus(),
      chainwork: info.chainwork.try_into().unwrap(),
      confirmations: info.confirmations,
      difficulty: info.difficulty,
      hash,
      feerate_percentiles: [
        stats.fee_rate_percentiles.fr_10th.to_sat(),
        stats.fee_rate_percentiles.fr_25th.to_sat(),
        stats.fee_rate_percentiles.fr_50th.to_sat(),
        stats.fee_rate_percentiles.fr_75th.to_sat(),
        stats.fee_rate_percentiles.fr_90th.to_sat(),
      ],
      height: info.height.try_into().unwrap(),
      max_fee: stats.max_fee.to_sat(),
      max_fee_rate: stats.max_fee_rate.to_sat(),
      max_tx_size: stats.max_tx_size,
      median_fee: stats.median_fee.to_sat(),
      median_time: info
        .median_time
        .map(|median_time| median_time.try_into().unwrap()),
      merkle_root: info.merkle_root,
      min_fee: stats.min_fee.to_sat(),
      min_fee_rate: stats.min_fee_rate.to_sat(),
      next_block: info.next_block_hash,
      nonce: info.nonce,
      previous_block: info.previous_block_hash,
      subsidy: stats.subsidy.to_sat(),
      target: target_as_block_hash(header.target()),
      timestamp: info.time.try_into().unwrap(),
      total_fee: stats.total_fee.to_sat(),
      total_size: stats.total_size,
      total_weight: stats.total_weight,
      transaction_count: info.n_tx.try_into().unwrap(),
      #[allow(clippy::cast_sign_loss)]
      version: info.version.to_consensus() as u32,
    }))
  })
}

pub(super) async fn blocktime_string(
  Extension(index): Extension<Arc<Index>>,
) -> ServerResult<String> {
  task::block_in_place(|| {
    Ok(
      index
        .block_time(index.block_height()?.ok_or_not_found(|| "blocktime")?)?
        .unix_timestamp()
        .to_string(),
    )
  })
}

pub(super) async fn children(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  children_paginated(Extension(index), Path((inscription_id, 0))).await
}

pub(super) async fn children_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  children_inscriptions_paginated(Extension(index), Path((inscription_id, 0))).await
}

pub(super) async fn children_inscriptions_paginated(
  Extension(index): Extension<Arc<Index>>,
  Path((parent, page)): Path<(InscriptionId, usize)>,
) -> ServerResult {
  task::block_in_place(|| {
    let parent_sequence_number = index
      .get_inscription_entry(parent)?
      .ok_or_not_found(|| format!("inscription {parent}"))?
      .sequence_number;

    let (ids, more) =
      index.get_children_by_sequence_number_paginated(parent_sequence_number, 100, page)?;

    let children = ids
      .into_iter()
      .map(|inscription_id| get_relative_inscription(&index, inscription_id))
      .collect::<ServerResult<Vec<api::RelativeInscriptionRecursive>>>()?;

    Ok(
      Json(api::ChildInscriptions {
        children,
        more,
        page,
      })
      .into_response(),
    )
  })
}

pub(super) async fn children_paginated(
  Extension(index): Extension<Arc<Index>>,
  Path((parent, page)): Path<(InscriptionId, usize)>,
) -> ServerResult {
  task::block_in_place(|| {
    let Some(parent) = index.get_inscription_entry(parent)? else {
      return Err(ServerError::NotFound(format!(
        "inscription {} not found",
        parent
      )));
    };

    let parent_sequence_number = parent.sequence_number;

    let (ids, more) =
      index.get_children_by_sequence_number_paginated(parent_sequence_number, 100, page)?;

    Ok(Json(api::Children { ids, more, page }).into_response())
  })
}

pub(super) async fn content(
  Extension(index): Extension<Arc<Index>>,
  Extension(settings): Extension<Arc<Settings>>,
  Extension(server_config): Extension<Arc<ServerConfig>>,
  Path(inscription_id): Path<InscriptionId>,
  accept_encoding: AcceptEncoding,
) -> ServerResult {
  task::block_in_place(|| {
    if settings.is_hidden(inscription_id) {
      return Ok(PreviewUnknownHtml.into_response());
    }

    let Some(mut inscription) = index.get_inscription_by_id(inscription_id)? else {
      return Err(ServerError::NotFound(format!(
        "inscription {inscription_id} not found"
      )));
    };

    if let Some(delegate) = inscription.delegate() {
      inscription = index
        .get_inscription_by_id(delegate)?
        .ok_or_not_found(|| format!("delegate {inscription_id}"))?
    }

    Ok(
      content_response(inscription, accept_encoding, &server_config)?
        .ok_or_not_found(|| format!("inscription {inscription_id} content"))?
        .into_response(),
    )
  })
}

pub(super) fn content_response(
  inscription: Inscription,
  accept_encoding: AcceptEncoding,
  server_config: &ServerConfig,
) -> ServerResult<Option<(HeaderMap, Vec<u8>)>> {
  let mut headers = HeaderMap::new();

  match &server_config.csp_origin {
    None => {
      headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self' 'unsafe-eval' 'unsafe-inline' data: blob:"),
      );
      headers.append(
          header::CONTENT_SECURITY_POLICY,
          HeaderValue::from_static("default-src *:*/content/ *:*/blockheight *:*/blockhash *:*/blockhash/ *:*/blocktime *:*/r/ 'unsafe-eval' 'unsafe-inline' data: blob:"),
        );
    }
    Some(origin) => {
      let csp = format!("default-src {origin}/content/ {origin}/blockheight {origin}/blockhash {origin}/blockhash/ {origin}/blocktime {origin}/r/ 'unsafe-eval' 'unsafe-inline' data: blob:");
      headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&csp).map_err(|err| ServerError::Internal(Error::from(err)))?,
      );
    }
  }

  headers.insert(
    header::CACHE_CONTROL,
    HeaderValue::from_static("public, max-age=1209600, immutable"),
  );

  headers.insert(
    header::CONTENT_TYPE,
    inscription
      .content_type()
      .and_then(|content_type| content_type.parse().ok())
      .unwrap_or(HeaderValue::from_static("application/octet-stream")),
  );

  if let Some(content_encoding) = inscription.content_encoding() {
    if accept_encoding.is_acceptable(&content_encoding) {
      headers.insert(header::CONTENT_ENCODING, content_encoding);
    } else if server_config.decompress && content_encoding == "br" {
      let Some(body) = inscription.into_body() else {
        return Ok(None);
      };

      let mut decompressed = Vec::new();

      Decompressor::new(body.as_slice(), 4096)
        .read_to_end(&mut decompressed)
        .map_err(|err| ServerError::Internal(err.into()))?;

      return Ok(Some((headers, decompressed)));
    } else {
      return Err(ServerError::NotAcceptable {
        accept_encoding,
        content_encoding,
      });
    }
  }

  let Some(body) = inscription.into_body() else {
    return Ok(None);
  };

  Ok(Some((headers, body)))
}

pub(super) async fn inscription(
  Extension(index): Extension<Arc<Index>>,
  Extension(server_config): Extension<Arc<ServerConfig>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  task::block_in_place(|| {
    let Some(inscription) = index.get_inscription_by_id(inscription_id)? else {
      return Err(ServerError::NotFound(format!(
        "inscription {} not found",
        inscription_id
      )));
    };

    let entry = index
      .get_inscription_entry(inscription_id)
      .unwrap()
      .unwrap();

    let satpoint = index
      .get_inscription_satpoint_by_id(inscription_id)
      .ok()
      .flatten()
      .unwrap();

    let output = if satpoint.outpoint == unbound_outpoint() {
      None
    } else {
      Some(
        index
          .get_transaction(satpoint.outpoint.txid)?
          .ok_or_not_found(|| format!("inscription {inscription_id} current transaction"))?
          .output
          .into_iter()
          .nth(satpoint.outpoint.vout.try_into().unwrap())
          .ok_or_not_found(|| format!("inscription {inscription_id} current transaction output"))?,
      )
    };

    let address = output.as_ref().and_then(|output| {
      server_config
        .chain
        .address_from_script(&output.script_pubkey)
        .ok()
        .map(|address| address.to_string())
    });

    Ok(
      Json(api::InscriptionRecursive {
        charms: Charm::charms(entry.charms),
        content_type: inscription.content_type().map(|s| s.to_string()),
        content_length: inscription.content_length(),
        delegate: inscription.delegate(),
        fee: entry.fee,
        height: entry.height,
        id: inscription_id,
        number: entry.inscription_number,
        output: satpoint.outpoint,
        value: output.as_ref().map(|o| o.value.to_sat()),
        sat: entry.sat,
        satpoint,
        timestamp: timestamp(entry.timestamp.into()).timestamp(),
        address,
      })
      .into_response(),
    )
  })
}

pub(super) async fn metadata(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  task::block_in_place(|| {
    let Some(inscription) = index.get_inscription_by_id(inscription_id)? else {
      return Err(ServerError::NotFound(format!(
        "inscription {} not found",
        inscription_id
      )));
    };

    let metadata = inscription
      .metadata
      .ok_or_not_found(|| format!("inscription {inscription_id} metadata"))?;

    Ok(Json(hex::encode(metadata)).into_response())
  })
}

pub(super) async fn parents(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  parents_paginated(Extension(index), Path((inscription_id, 0))).await
}

pub async fn parent_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(inscription_id): Path<InscriptionId>,
) -> ServerResult {
  parent_inscriptions_paginated(Extension(index), Path((inscription_id, 0))).await
}

pub async fn parent_inscriptions_paginated(
  Extension(index): Extension<Arc<Index>>,
  Path((child, page)): Path<(InscriptionId, usize)>,
) -> ServerResult {
  task::block_in_place(|| {
    let entry = index
      .get_inscription_entry(child)?
      .ok_or_not_found(|| format!("inscription {child}"))?;

    let (ids, more) = index.get_parents_by_sequence_number_paginated(entry.parents, 100, page)?;

    let parents = ids
      .into_iter()
      .map(|inscription_id| get_relative_inscription(&index, inscription_id))
      .collect::<ServerResult<Vec<api::RelativeInscriptionRecursive>>>()?;

    Ok(
      Json(api::ParentInscriptions {
        parents,
        more,
        page,
      })
      .into_response(),
    )
  })
}

pub(super) async fn parents_paginated(
  Extension(index): Extension<Arc<Index>>,
  Path((inscription_id, page)): Path<(InscriptionId, usize)>,
) -> ServerResult {
  task::block_in_place(|| {
    let child = index
      .get_inscription_entry(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    let (ids, more) = index.get_parents_by_sequence_number_paginated(child.parents, 100, page)?;

    let page_index =
      u32::try_from(page).map_err(|_| anyhow!("page index {} out of range", page))?;

    Ok(
      Json(api::Inscriptions {
        ids,
        more,
        page_index,
      })
      .into_response(),
    )
  })
}

pub(super) async fn sat(
  Extension(index): Extension<Arc<Index>>,
  Path(sat): Path<u64>,
) -> ServerResult<Json<api::SatInscriptions>> {
  sat_paginated(Extension(index), Path((sat, 0))).await
}

pub(super) async fn sat_at_index(
  Extension(index): Extension<Arc<Index>>,
  Path((DeserializeFromStr(sat), inscription_index)): Path<(DeserializeFromStr<Sat>, isize)>,
) -> ServerResult<Json<api::SatInscription>> {
  task::block_in_place(|| {
    if !index.has_sat_index() {
      return Err(ServerError::NotFound(
        "this server has no sat index".to_string(),
      ));
    }

    let id = index.get_inscription_id_by_sat_indexed(sat, inscription_index)?;

    Ok(Json(api::SatInscription { id }))
  })
}

pub(super) async fn sat_paginated(
  Extension(index): Extension<Arc<Index>>,
  Path((sat, page)): Path<(u64, u64)>,
) -> ServerResult<Json<api::SatInscriptions>> {
  task::block_in_place(|| {
    if !index.has_sat_index() {
      return Err(ServerError::NotFound("this server has no sat index".into()));
    }

    let (ids, more) = index.get_inscription_ids_by_sat_paginated(Sat(sat), 100, page)?;

    Ok(Json(api::SatInscriptions { ids, more, page }))
  })
}

pub(super) async fn sat_at_index_content(
  index: Extension<Arc<Index>>,
  settings: Extension<Arc<Settings>>,
  server_config: Extension<Arc<ServerConfig>>,
  Path((DeserializeFromStr(sat), inscription_index)): Path<(DeserializeFromStr<Sat>, isize)>,
  accept_encoding: AcceptEncoding,
) -> ServerResult {
  let inscription_id = task::block_in_place(|| {
    if !index.has_sat_index() {
      return Err(ServerError::NotFound("this server has no sat index".into()));
    }

    index
      .get_inscription_id_by_sat_indexed(sat, inscription_index)?
      .ok_or_not_found(|| format!("inscription on sat {sat}"))
  })?;

  content(
    index,
    settings,
    server_config,
    Path(inscription_id),
    accept_encoding,
  )
  .await
}

fn get_relative_inscription(
  index: &Index,
  id: InscriptionId,
) -> ServerResult<api::RelativeInscriptionRecursive> {
  let entry = index
    .get_inscription_entry(id)?
    .ok_or_not_found(|| format!("inscription {id}"))?;

  let satpoint = index
    .get_inscription_satpoint_by_id(id)?
    .ok_or_not_found(|| format!("satpoint for inscription {id}"))?;

  Ok(api::RelativeInscriptionRecursive {
    charms: Charm::charms(entry.charms),
    fee: entry.fee,
    height: entry.height,
    id,
    number: entry.inscription_number,
    output: satpoint.outpoint,
    sat: entry.sat,
    satpoint,
    timestamp: timestamp(entry.timestamp.into()).timestamp(),
  })
}

pub(super) async fn tx(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<Txid>,
) -> ServerResult<Json<String>> {
  task::block_in_place(|| {
    Ok(Json(
      index
        .get_transaction_hex_recursive(txid)?
        .ok_or_not_found(|| format!("transaction {txid}"))?,
    ))
  })
}

pub(super) async fn undelegated_content(
  Extension(index): Extension<Arc<Index>>,
  Extension(settings): Extension<Arc<Settings>>,
  Extension(server_config): Extension<Arc<ServerConfig>>,
  Path(inscription_id): Path<InscriptionId>,
  accept_encoding: AcceptEncoding,
) -> ServerResult {
  task::block_in_place(|| {
    if settings.is_hidden(inscription_id) {
      return Ok(PreviewUnknownHtml.into_response());
    }

    let inscription = index
      .get_inscription_by_id(inscription_id)?
      .ok_or_not_found(|| format!("inscription {inscription_id}"))?;

    Ok(
      r::content_response(inscription, accept_encoding, &server_config)?
        .ok_or_not_found(|| format!("inscription {inscription_id} content"))?
        .into_response(),
    )
  })
}

pub(super) async fn utxo(
  Extension(index): Extension<Arc<Index>>,
  Path(outpoint): Path<OutPoint>,
) -> ServerResult {
  task::block_in_place(|| {
    Ok(
      Json(
        index
          .get_utxo_recursive(outpoint)?
          .ok_or_not_found(|| format!("output {outpoint}"))?,
      )
      .into_response(),
    )
  })
}
