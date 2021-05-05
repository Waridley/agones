// Copyright 2018 Google LLC All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod proto {
	#[cfg(feature = "sdk")]
	pub mod sdk {
		tonic::include_proto!("agones.dev.sdk");
	}
	#[cfg(feature = "alpha")]
	pub mod alpha {
		tonic::include_proto!("agones.dev.sdk.alpha");
	}
	#[cfg(feature = "beta")]
	pub mod beta {
		tonic::include_proto!("agones.dev.sdk.beta");
	}
	#[cfg(feature = "allocation")]
	pub mod allocation {
		tonic::include_proto!("allocation");
	}
}

pub mod sdk {
	#[cfg(feature = "sdk")]
	pub use crate::proto::sdk::{
		Empty, KeyValue, Duration,
	};
	#[cfg(feature = "sdk")]
	use crate::proto::sdk::sdk_client::SdkClient;
	#[cfg(feature = "alpha")]
	use crate::proto::alpha;
	
	use tonic::{
		Response, Status, IntoRequest,
		codegen::StdError,
		transport::{Channel, Endpoint, Error,}
	};
	
	pub struct Sdk {
		#[cfg(feature = "sdk")]
		stable: SdkClient<Channel>,
		#[cfg(feature = "alpha")]
		alpha: alpha::sdk_client::SdkClient<Channel>,
	}
	
	impl Sdk {
		pub fn new<D>(dst: D) -> Result<Self, tonic::transport::Error>
		where
			D: std::convert::TryInto<tonic::transport::Endpoint>,
			D::Error: Into<StdError>,
		{
			let conn = Endpoint::new(dst)?.connect_lazy()?;
			Ok(Sdk {
				#[cfg(feature = "sdk")]
				stable: SdkClient::new(conn.clone()),
				#[cfg(feature = "alpha")]
				alpha: alpha::sdk_client::SdkClient::new(conn),
			})
		}
	}
	
	#[cfg(feature = "sdk")]
	impl Sdk {
		/// Call when the GameServer is ready
		pub async fn ready(
			&mut self,
			request: impl IntoRequest<Empty>,
		) -> Result<Response<Empty>, Status> {
			self.stable.ready(request).await
		}
		
		pub async fn shutdown(
			&mut self,
			request: impl IntoRequest<Empty>,
		) -> Result<Response<Empty>, Status> {
			self.stable.shutdown(request).await
		}
		
		pub async fn set_label(
			&mut self,
			request: impl tonic::IntoRequest<KeyValue>,
		) -> Result<Response<Empty>, Status> {
			self.stable.set_label(request).await
		}
		
		pub async fn set_annotation(
			&mut self,
			request: impl tonic::IntoRequest<KeyValue>,
		) -> Result<Response<Empty>, Status> {
			self.stable.set_annotation(request).await
		}
		
		pub async fn allocate(
			&mut self,
			request: impl tonic::IntoRequest<Empty>,
		) -> Result<Response<Empty>, Status> {
			self.stable.allocate(request).await
		}
		
		pub async fn reserve(
			&mut self,
			request: impl tonic::IntoRequest<Duration>,
		) -> Result<Response<Empty>, Status> {
			self.stable.reserve(request).await
		}
	}
	
	#[cfg(feature = "alpha")]
	impl Sdk {
		pub async fn set_player_capacity(
			&mut self,
			request: impl tonic::IntoRequest<alpha::Count>,
		) -> Result<Response<alpha::Empty>, Status> {
			self.alpha.set_player_capacity(request).await
		}
		
		pub async fn player_connect(
			&mut self,
			request: impl tonic::IntoRequest<alpha::PlayerId>,
		) -> Result<Response<alpha::Bool>, Status> {
			self.alpha.player_connect(request).await
		}
		
		pub async fn player_disconnect(
			&mut self,
			request: impl tonic::IntoRequest<alpha::PlayerId>,
		) -> Result<Response<alpha::Bool>, Status> {
			self.alpha.player_disconnect(request).await
		}
	}
}