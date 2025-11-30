let api_key = &self.config.breez_api_key;

// Create new Greenlight node
let response = self.client.create_node(api_key, ...);

// Open first channel with 200,000 sats inbound liquidity
let channel_response = self.client.open_channel(api_key, ...);

// Ensure response includes invite_code, node_id, and initial_channel_opened
let response_data = json!({
    "invite_code": channel_response.invite_code,
    "node_id": channel_response.node_id,
    "initial_channel_opened": true,
});