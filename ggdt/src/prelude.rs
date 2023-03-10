// to get everything this library has to offer, including all `SystemResources` implementations

pub use crate::{
	*,
	audio::prelude::*,
	base::*,
	entities::*,
	events::*,
	graphics::prelude::*,
	math::prelude::*,
	states::*,
	system::{
		prelude::*,
		res::{
			dos_like::*,
		},
	},
	utils::prelude::*,
};

// specific module preludes that can be used instead that grab everything relevant to a specific `SystemResources`
// implementation only, since most applications will only use one and not care about the rest

pub mod dos_like {
	pub use crate::{
		*,
		audio::prelude::*,
		base::*,
		entities::*,
		events::*,
		graphics::indexed::prelude::*,
		math::prelude::*,
		states::*,
		system::{
			prelude::*,
			res::{
				dos_like::*,
			},
		},
		utils::prelude::*,
	};
}
